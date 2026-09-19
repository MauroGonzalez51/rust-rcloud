//! Mirror of `src/config/registry.rs`.
//!
//! Exercises the public `Registry` API: `load` (missing/empty/valid/corrupt)
//! and `tx` (persist + in-memory rollback on save failure).

use rcloud::Registry;
use std::fs;

#[test]
fn load_missing_file_creates_empty_registry() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("registry.json");

    let registry = Registry::load(&path)?;

    assert!(registry.remotes.is_empty());
    assert!(registry.paths.is_empty());
    // The file is created on load.
    assert!(path.exists());
    Ok(())
}

#[test]
fn load_empty_file_creates_empty_registry() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("registry.json");
    fs::write(&path, "   \n")?;

    let registry = Registry::load(&path)?;
    assert!(registry.remotes.is_empty());
    assert!(registry.paths.is_empty());
    Ok(())
}

#[test]
fn load_corrupt_file_errors() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("registry.json");
    fs::write(&path, "{ this is not valid json ")?;

    assert!(Registry::load(&path).is_err());
    Ok(())
}

#[test]
fn tx_persists_changes_and_reloads() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("registry.json");

    let mut registry = Registry::load(&path)?;
    registry.tx(|rgx| {
        rgx.remotes.push(rcloud::Remote {
            id: "id-1".into(),
            remote_name: "drive".into(),
            provider: "drive".into(),
        });
    })?;

    // Re-load from disk to confirm it was written.
    let reloaded = Registry::load(&path)?;
    assert_eq!(reloaded.remotes.len(), 1);
    assert_eq!(reloaded.remotes[0].id, "id-1");
    Ok(())
}

#[test]
fn tx_rolls_back_in_memory_when_save_fails() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("registry.json");
    let mut registry = Registry::load(&path)?;

    // Point the registry at a path that cannot be written (a directory), so
    // save() fails and tx() restores the prior in-memory state.
    let unwritable = dir.path().join("as-dir");
    fs::create_dir(&unwritable)?;
    registry.registry_path = unwritable;

    registry.tx(|rgx| {
        rgx.remotes.push(rcloud::Remote {
            id: "ghost".into(),
            remote_name: "x".into(),
            provider: "x".into(),
        });
    })?;

    // Save failed, so the push must have been rolled back.
    assert!(
        registry.remotes.is_empty(),
        "expected rollback to discard the failed mutation"
    );
    Ok(())
}
