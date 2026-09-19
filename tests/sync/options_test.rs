//! Mirror of `src/cli/commands/sync/utils/options.rs`.
//!
//! `force` decides whether a sync proceeds based on the stored vs computed
//! hash and the `--force` flag; `clean` optionally clears the local
//! destination before a pull.

use rcloud::cli::commands::sync::utils::{ForceResult, clean, force};
use rcloud::{HookExecType, PathConfig, PathConfigHooks};
use std::fs;

fn path_with(hash: Option<&str>, local_path: &str) -> PathConfig {
    PathConfig {
        id: String::new(),
        remote_id: String::new(),
        local_path: local_path.to_string(),
        remote_path: String::new(),
        hash: hash.map(str::to_string),
        remote_filename: None,
        tags: vec![],
        hooks: PathConfigHooks {
            push: vec![],
            pull: vec![],
        },
    }
}

// --- force: Push ---

#[test]
fn push_skips_when_hash_matches_and_not_forced() {
    let cfg = path_with(Some("abc"), "");
    assert_eq!(
        force(&HookExecType::Push, &false, &cfg, "abc"),
        ForceResult::HashMatch
    );
}

#[test]
fn push_proceeds_when_hash_differs() {
    let cfg = path_with(Some("abc"), "");
    assert_eq!(
        force(&HookExecType::Push, &false, &cfg, "def"),
        ForceResult::Proceed
    );
}

#[test]
fn push_proceeds_when_forced_even_if_hash_matches() {
    let cfg = path_with(Some("abc"), "");
    assert_eq!(
        force(&HookExecType::Push, &true, &cfg, "abc"),
        ForceResult::Proceed
    );
}

#[test]
fn push_proceeds_when_no_stored_hash() {
    let cfg = path_with(None, "");
    assert_eq!(
        force(&HookExecType::Push, &false, &cfg, "abc"),
        ForceResult::Proceed
    );
}

// --- force: Pull ---

#[test]
fn pull_skips_when_hash_matches_and_local_exists() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let local = dir.path().join("data");
    fs::create_dir(&local)?;
    let cfg = path_with(Some("abc"), local.to_str().unwrap());

    assert_eq!(
        force(&HookExecType::Pull, &false, &cfg, "abc"),
        ForceResult::HashMatch
    );
    Ok(())
}

#[test]
fn pull_reports_path_not_found_when_hash_matches_but_local_missing() {
    let cfg = path_with(Some("abc"), "/definitely/not/here/rcloud-xyz");
    assert_eq!(
        force(&HookExecType::Pull, &false, &cfg, "abc"),
        ForceResult::PathNotFound
    );
}

#[test]
fn pull_proceeds_when_hash_differs() {
    let cfg = path_with(Some("abc"), "/tmp");
    assert_eq!(
        force(&HookExecType::Pull, &false, &cfg, "def"),
        ForceResult::Proceed
    );
}

#[test]
fn pull_proceeds_when_forced() {
    let cfg = path_with(Some("abc"), "/tmp");
    assert_eq!(
        force(&HookExecType::Pull, &true, &cfg, "abc"),
        ForceResult::Proceed
    );
}

// --- clean ---

#[test]
fn clean_removes_local_dir_on_pull_when_requested() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let local = dir.path().join("target");
    fs::create_dir(&local)?;
    fs::write(local.join("f.txt"), b"x")?;

    clean(&HookExecType::Pull, &true, local.to_str().unwrap())?;
    assert!(!local.exists());
    Ok(())
}

#[test]
fn clean_is_noop_on_pull_when_not_requested() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let local = dir.path().join("target");
    fs::create_dir(&local)?;

    clean(&HookExecType::Pull, &false, local.to_str().unwrap())?;
    assert!(local.exists());
    Ok(())
}

#[test]
fn clean_is_ignored_on_push() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let local = dir.path().join("target");
    fs::create_dir(&local)?;

    // Even with clean=true, push must not remove anything.
    clean(&HookExecType::Push, &true, local.to_str().unwrap())?;
    assert!(local.exists());
    Ok(())
}

#[test]
fn clean_missing_path_on_pull_is_ok() -> anyhow::Result<()> {
    clean(&HookExecType::Pull, &true, "/tmp/rcloud-nonexistent-xyz")?;
    Ok(())
}
