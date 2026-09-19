//! Mirror of `src/utils/hash.rs`.
//!
//! Exercises the public `Hash` API: `hash_bytes` and `hash_path` (files and
//! directories). The private `hash_file`/`hash_directory` are covered
//! transitively through `hash_path`.

use rcloud::utils::Hash;
use std::fs;

#[test]
fn hash_bytes_is_deterministic() {
    let a = Hash::hash_bytes(b"hello world");
    let b = Hash::hash_bytes(b"hello world");
    assert_eq!(a, b);
}

#[test]
fn hash_bytes_differs_for_different_input() {
    assert_ne!(Hash::hash_bytes(b"a"), Hash::hash_bytes(b"b"));
}

#[test]
fn hash_bytes_is_sha256_hex() {
    // Known SHA-256 of the empty input.
    assert_eq!(
        Hash::hash_bytes(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn hash_path_file_matches_hash_bytes() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("f.txt");
    fs::write(&file, b"payload")?;

    assert_eq!(Hash::hash_path(&file)?, Hash::hash_bytes(b"payload"));
    Ok(())
}

#[test]
fn hash_path_file_is_deterministic() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("f.txt");
    fs::write(&file, b"same content")?;

    assert_eq!(Hash::hash_path(&file)?, Hash::hash_path(&file)?);
    Ok(())
}

#[test]
fn hash_path_directory_is_deterministic() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("tree");
    fs::create_dir(&root)?;
    fs::write(root.join("a.txt"), b"alpha")?;
    fs::write(root.join("b.txt"), b"beta")?;

    assert_eq!(Hash::hash_path(&root)?, Hash::hash_path(&root)?);
    Ok(())
}

#[test]
fn hash_path_directory_changes_with_content() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;

    let root_a = dir.path().join("a");
    fs::create_dir(&root_a)?;
    fs::write(root_a.join("f.txt"), b"one")?;

    let root_b = dir.path().join("b");
    fs::create_dir(&root_b)?;
    fs::write(root_b.join("f.txt"), b"two")?;

    assert_ne!(Hash::hash_path(&root_a)?, Hash::hash_path(&root_b)?);
    Ok(())
}

#[test]
fn hash_path_missing_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist");
    assert!(Hash::hash_path(&missing).is_err());
}
