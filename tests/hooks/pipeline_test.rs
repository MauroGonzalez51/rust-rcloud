//! Hook pipeline round-trip tests.
//!
//! Exercises `execute_hooks` the way the sync engine does: push hooks run in
//! order, pull hooks run in reverse. A push followed by the matching pull must
//! reproduce the original content byte for byte.
//!
//! Encryption is covered by unit tests in `encryption_hook.rs` instead, since
//! its `process` prompts for a password interactively and cannot run headless.

use anyhow::Context;
use rcloud::cli::commands::sync::utils::execute_hooks;
use rcloud::{
    AppConfig, HookConfig, HookContext, HookExecType, PathConfig, PathConfigHooks, Remote,
    ZipHookConfig,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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

/// Runs push hooks in order, then pull hooks in reverse (as the sync engine
/// does), returning the final path produced by the pull side.
fn round_trip(
    start: &Path,
    push: &[HookConfig],
    pull: &[HookConfig],
) -> anyhow::Result<std::path::PathBuf> {
    let cfg = AppConfig::default();

    let pushed = execute_hooks(
        HookContext::new(start.to_path_buf(), "", &mock_remote(), &mock_path()),
        push,
        &cfg,
    )
    .context("push pipeline failed")?;

    let reversed_pull: Vec<HookConfig> = pull.iter().rev().cloned().collect();

    let pulled = execute_hooks(
        HookContext::new(pushed.path.clone(), "", &mock_remote(), &mock_path()),
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

#[test]
fn zip_round_trip_single_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("note.txt");
    let contents = b"round trip me";
    fs::write(&src, contents)?;

    let result = round_trip(&src, &[zip_push()], &[zip_pull()])?;

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

    let result = round_trip(&src, &[zip_push()], &[zip_pull()])?;

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
        HookContext::new(src.clone(), "", &mock_remote(), &mock_path()),
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
