//! Hook pipeline round-trip tests.
//!
//! Exercises `execute_hooks` the way the sync engine does: push hooks run in
//! order, pull hooks run in reverse. A push followed by the matching pull must
//! reproduce the original content byte for byte.
//!
//! Encryption now runs through the real `execute_hooks` engine: the password
//! prompt is injected via a `MockPasswordProvider` in the `HookDependencies`,
//! so the full `Hook::process` path (acquire password -> verify -> transform)
//! is exercised headless, not bypassed.

use crate::support::{test_dependencies, MockPasswordProvider, MockRcloneRunner};
use anyhow::Context;
use rcloud::cli::commands::sync::utils::execute_hooks;
use rcloud::{
    AppConfig, EncryptionHook, EncryptionHookConfig, Hook, HookConfig, HookContext,
    HookDependencies, HookExecType, PathConfig, PathConfigHooks, Remote, ZipHook, ZipHookConfig,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Fixed password the mock provider hands to the encryption hook.
const TEST_PASSWORD: &str = "correct horse battery staple";

fn mock_remote() -> Remote {
    Remote {
        id: String::new(),
        remote_name: String::from("drive"),
        provider: String::from("drive"),
    }
}

fn mock_path() -> PathConfig {
    PathConfig {
        id: String::new(),
        remote_id: String::new(),
        local_path: String::new(),
        remote_path: String::new(),
        hash: None,
        remote_filename: None,
        tags: vec![],
        hooks: PathConfigHooks {
            push: vec![],
            pull: vec![],
        },
    }
}

/// Dependencies whose password provider returns `password`.
fn deps_with_password(password: &str) -> HookDependencies {
    test_dependencies(
        Arc::new(MockRcloneRunner::new()),
        Arc::new(MockPasswordProvider::new(password)),
    )
}

fn zip_push() -> HookConfig {
    HookConfig::Zip(ZipHookConfig {
        exec: HookExecType::Push,
        level: Some(6),
        exclude: None,
    })
}

fn zip_pull() -> HookConfig {
    HookConfig::Zip(ZipHookConfig {
        exec: HookExecType::Pull,
        level: None,
        exclude: None,
    })
}

/// Encryption hook configs for a given stored `hash`.
fn encrypt_push(hash: &str) -> HookConfig {
    HookConfig::Encryption(EncryptionHookConfig {
        exec: HookExecType::Push,
        hash: hash.to_string(),
    })
}

fn encrypt_pull(hash: &str) -> HookConfig {
    HookConfig::Encryption(EncryptionHookConfig {
        exec: HookExecType::Pull,
        hash: hash.to_string(),
    })
}

/// Runs push hooks in order, then pull hooks in reverse (as the sync engine
/// does), returning the final path produced by the pull side. `password` is
/// injected into the dependency set for any encryption legs.
fn round_trip(
    start: &Path,
    push: &[HookConfig],
    pull: &[HookConfig],
    password: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let cfg = AppConfig::default();

    let pushed = execute_hooks(
        HookContext::new(
            start.to_path_buf(),
            deps_with_password(password),
            &mock_remote(),
            &mock_path(),
        ),
        push,
        &cfg,
    )
    .context("push pipeline failed")?;

    let reversed_pull: Vec<HookConfig> = pull.iter().rev().cloned().collect();

    let pulled = execute_hooks(
        HookContext::new(
            pushed.path.clone(),
            deps_with_password(password),
            &mock_remote(),
            &mock_path(),
        ),
        &reversed_pull,
        &cfg,
    )
    .context("pull pipeline failed")?;

    Ok(pulled.path)
}

/// Reads every file under `root` into a map of relative-path -> bytes.
fn read_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        out.insert(rel, fs::read(entry.path()).unwrap());
    }
    out
}

// --- Zip-only pipelines ----------------------------------------------------

#[test]
fn zip_round_trip_single_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("note.txt");
    let contents = b"round trip me";
    fs::write(&src, contents)?;

    let result = round_trip(&src, &[zip_push()], &[zip_pull()], "")?;

    // Zip pull extracts into a directory; the single file lands inside it.
    let extracted = result.join("note.txt");
    assert!(extracted.exists(), "extracted file missing at {result:?}");
    assert_eq!(fs::read(&extracted)?, contents);
    Ok(())
}

#[test]
fn zip_round_trip_directory_preserves_all_files() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("payload");
    fs::create_dir(&src)?;
    fs::write(src.join("a.txt"), b"alpha")?;
    fs::write(src.join("b.txt"), b"beta")?;
    let nested = src.join("nested");
    fs::create_dir(&nested)?;
    fs::write(nested.join("c.txt"), b"gamma")?;

    let original = read_tree(&src);

    let result = round_trip(&src, &[zip_push()], &[zip_pull()], "")?;

    let restored = read_tree(&result);
    assert_eq!(
        restored, original,
        "directory content changed across zip round-trip"
    );
    Ok(())
}

#[test]
fn zip_push_changes_path_and_records_checksum() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.bin");
    fs::write(&src, vec![1u8, 2, 3, 4, 5])?;

    let cfg = AppConfig::default();
    let pushed = execute_hooks(
        HookContext::new(
            src.clone(),
            deps_with_password(""),
            &mock_remote(),
            &mock_path(),
        ),
        &[zip_push()],
        &cfg,
    )?;

    // The path must have moved to the produced archive, not the source.
    assert_ne!(pushed.path, src);
    assert!(pushed.path.exists());
    assert!(
        pushed
            .metadata
            .contains_key(&rcloud::HookContextMetadata::ZipChecksum)
    );
    Ok(())
}

// --- Mixed pipelines: Zip + Encryption -------------------------------------
//
// The sync engine runs push hooks in order and pull hooks in reverse. For a
// [Zip, Encryption] push pipeline the real order is:
//   push:  zip(source) -> encrypt(archive)
//   pull:  decrypt(blob) -> unzip(archive)   (reverse)
// A full round trip through `execute_hooks` must reproduce the original bytes.

#[test]
fn zip_encrypt_round_trip_single_file() -> anyhow::Result<()> {
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("secret.txt");
    let contents = b"compress then encrypt then reverse";
    fs::write(&src, contents)?;

    let result = round_trip(
        &src,
        &[zip_push(), encrypt_push(&hash)],
        &[zip_pull(), encrypt_pull(&hash)],
        TEST_PASSWORD,
    )?;

    // After the reversed pull (decrypt -> unzip) the single file lands inside
    // the extracted directory.
    let restored = result.join("secret.txt");
    assert!(restored.exists(), "restored file missing at {result:?}");
    assert_eq!(fs::read(&restored)?, contents);
    Ok(())
}

#[test]
fn zip_encrypt_round_trip_directory_preserves_all_files() -> anyhow::Result<()> {
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("payload");
    fs::create_dir(&src)?;
    fs::write(src.join("a.txt"), b"alpha")?;
    fs::write(src.join("b.txt"), b"beta")?;
    let nested = src.join("nested");
    fs::create_dir(&nested)?;
    fs::write(nested.join("c.txt"), b"gamma")?;

    let original = read_tree(&src);

    let result = round_trip(
        &src,
        &[zip_push(), encrypt_push(&hash)],
        &[zip_pull(), encrypt_pull(&hash)],
        TEST_PASSWORD,
    )?;

    let restored = read_tree(&result);
    assert_eq!(
        restored, original,
        "directory content changed across zip+encrypt round-trip"
    );
    Ok(())
}

#[test]
fn encrypt_only_round_trip_single_file() -> anyhow::Result<()> {
    // Encryption alone (no zip) must also round-trip through the engine.
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("plain.txt");
    let contents = b"just encryption, no compression";
    fs::write(&src, contents)?;

    let result = round_trip(
        &src,
        &[encrypt_push(&hash)],
        &[encrypt_pull(&hash)],
        TEST_PASSWORD,
    )?;

    assert!(result.exists(), "decrypted file missing at {result:?}");
    assert_eq!(fs::read(&result)?, contents);
    Ok(())
}

#[test]
fn zip_encrypt_intermediate_is_encrypted_not_plain_zip() -> anyhow::Result<()> {
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let cfg = AppConfig::default();
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.txt");
    fs::write(&src, b"payload bytes to protect")?;

    // Push the full [zip, encrypt] pipeline through the engine.
    let pushed = execute_hooks(
        HookContext::new(
            src.clone(),
            deps_with_password(TEST_PASSWORD),
            &mock_remote(),
            &mock_path(),
        ),
        &[zip_push(), encrypt_push(&hash)],
        &cfg,
    )?;

    // The engine output is the encrypted blob: it must carry the marker header
    // and not look like a bare zip.
    let cipher = fs::read(&pushed.path)?;
    assert!(
        cipher.starts_with(b"RCLOUD_ENCRYPTED"),
        "final pushed output missing encryption marker header"
    );
    Ok(())
}

#[test]
fn zip_encrypt_wrong_password_fails_pull() -> anyhow::Result<()> {
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let cfg = AppConfig::default();
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.txt");
    fs::write(&src, b"only the right key unlocks this")?;

    // Push with the correct password.
    let pushed = execute_hooks(
        HookContext::new(
            src.clone(),
            deps_with_password(TEST_PASSWORD),
            &mock_remote(),
            &mock_path(),
        ),
        &[zip_push(), encrypt_push(&hash)],
        &cfg,
    )?;

    // Pull with the wrong password: verify() must reject it before any
    // decryption, aborting the reversed pipeline.
    let reversed: Vec<HookConfig> = [zip_pull(), encrypt_pull(&hash)]
        .iter()
        .rev()
        .cloned()
        .collect();

    let result = execute_hooks(
        HookContext::new(
            pushed.path.clone(),
            deps_with_password("wrong-password"),
            &mock_remote(),
            &mock_path(),
        ),
        &reversed,
        &cfg,
    );
    assert!(result.is_err(), "wrong password unexpectedly decrypted");
    Ok(())
}

// --- Pure `transform` phase, exercised without any mocks -------------------
//
// The point of the acquire/transform split: the pure transform can be driven
// directly by passing the acquired value as data. No password provider, no
// rclone runner needed — the dependencies never get touched.

/// Dependencies whose providers must never be called (they panic if they are),
/// proving `transform` does not reach into the outside world.
fn unused_dependencies() -> HookDependencies {
    struct PanicPassword;
    impl rcloud::PasswordProvider for PanicPassword {
        fn get(&self) -> anyhow::Result<String> {
            panic!("transform must not acquire a password");
        }
    }
    struct PanicRclone;
    impl rcloud::RcloneRunner for PanicRclone {
        fn transfer(&self, _: &str, _: &str) -> anyhow::Result<std::process::ExitStatus> {
            panic!("transform must not run rclone");
        }
        fn copy_to(&self, _: &str, _: &str) -> anyhow::Result<bool> {
            panic!("transform must not run rclone");
        }
        fn list(&self, _: &str) -> anyhow::Result<Vec<String>> {
            panic!("transform must not run rclone");
        }
        fn purge(&self, _: &str) -> anyhow::Result<()> {
            panic!("transform must not run rclone");
        }
    }
    test_dependencies(Arc::new(PanicRclone), Arc::new(PanicPassword))
}

#[test]
fn zip_transform_runs_without_touching_dependencies() -> anyhow::Result<()> {
    let cfg = AppConfig::default();
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.txt");
    fs::write(&src, b"pure transform, no mocks")?;

    let hook = ZipHook::from(ZipHookConfig {
        exec: HookExecType::Push,
        level: Some(6),
        exclude: None,
    });

    let ctx = HookContext::new(src, unused_dependencies(), &mock_remote(), &mock_path());

    // Zip's Acquired is (): pass it directly, no acquire phase, no mocks.
    let out = hook.transform(ctx, &cfg, ())?;
    assert!(out.path.exists());
    assert!(
        out.metadata
            .contains_key(&rcloud::HookContextMetadata::ZipChecksum)
    );
    Ok(())
}

#[test]
fn encryption_transform_is_pure_given_the_password() -> anyhow::Result<()> {
    // Drive encrypt then decrypt purely through `transform`, feeding the
    // password as the acquired value. The password provider is never invoked.
    let cfg = AppConfig::default();
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("plain.txt");
    let contents = b"transform-only round trip";
    fs::write(&src, contents)?;

    let enc = EncryptionHook {
        exec: HookExecType::Push,
        hash: hash.clone(),
    };
    let encrypted = enc.transform(
        HookContext::new(src, unused_dependencies(), &mock_remote(), &mock_path()),
        &cfg,
        TEST_PASSWORD.to_string(),
    )?;

    let dec = EncryptionHook {
        exec: HookExecType::Pull,
        hash,
    };
    let decrypted = dec.transform(
        HookContext::new(
            encrypted.path.clone(),
            unused_dependencies(),
            &mock_remote(),
            &mock_path(),
        ),
        &cfg,
        TEST_PASSWORD.to_string(),
    )?;

    assert_eq!(fs::read(&decrypted.path)?, contents);
    Ok(())
}

#[test]
fn encryption_acquire_rejects_wrong_password() -> anyhow::Result<()> {
    // The impure acquire phase: with a provider returning the wrong password,
    // verify() must fail before any transform happens.
    let hash = EncryptionHookConfig::derive_hash(TEST_PASSWORD)?;
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("plain.txt");
    fs::write(&src, b"x")?;

    let enc = EncryptionHook {
        exec: HookExecType::Push,
        hash,
    };
    let ctx = HookContext::new(
        src,
        deps_with_password("wrong"),
        &mock_remote(),
        &mock_path(),
    );

    assert!(enc.acquire(&ctx).is_err(), "acquire accepted wrong password");
    Ok(())
}
