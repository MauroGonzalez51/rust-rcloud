//! Backup hook end-to-end tests.
//!
//! The backup hook shells out to `rclone` for its remote replicas, so these
//! tests inject a `MockRcloneRunner` via `HookDependencies` and assert on the
//! recorded rclone calls (list -> purge -> copy_to) instead of touching a real
//! remote.

use crate::support::{test_dependencies, MockPasswordProvider, MockRcloneRunner};
use rcloud::{
    AppConfig, BackupHook, BackupType, Hook, HookContext, HookDependencies, HookExecType,
    PathConfig, PathConfigHooks, Remote,
};
use std::sync::Arc;

fn mock_remote() -> Remote {
    Remote {
        id: String::new(),
        remote_name: String::from("drive"),
        provider: String::from("drive"),
    }
}

fn mock_path(remote_path: &str) -> PathConfig {
    PathConfig {
        id: String::new(),
        remote_id: String::new(),
        local_path: String::new(),
        remote_path: remote_path.to_string(),
        hash: None,
        remote_filename: None,
        tags: vec![],
        hooks: PathConfigHooks {
            push: vec![],
            pull: vec![],
        },
    }
}

fn deps_with(rclone: Arc<MockRcloneRunner>) -> HookDependencies {
    test_dependencies(rclone, Arc::new(MockPasswordProvider::new("")))
}

/// A remote backup with one existing replica and `replicas = 1` must:
/// list the replicas, purge the oldest, then copy the new one.
#[test]
fn remote_backup_lists_rotates_and_copies() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.txt");
    std::fs::write(&src, b"backup me")?;

    // One existing replica so rotation (purge) kicks in at replicas = 1.
    let runner = Arc::new(MockRcloneRunner::new().with_list(vec!["100.1".to_string()]));

    let hook = BackupHook {
        exec: HookExecType::Push,
        types: vec![BackupType::Remote],
        local_path: None,
        remote_path: Some("backups/data".to_string()),
        replicas: 1,
    };

    let ctx = HookContext::new(
        src.clone(),
        deps_with(Arc::clone(&runner)),
        &mock_remote(),
        &mock_path("remote/data.txt"),
    );

    // Backup does not change the path.
    let result = hook.process(ctx, &AppConfig::default())?;
    assert_eq!(result.path, src, "backup hook must not change the path");

    let calls = runner.recorded();
    let methods: Vec<&str> = calls.iter().map(|c| c[0].as_str()).collect();

    assert!(methods.contains(&"list"), "expected a list call: {calls:?}");
    assert!(
        methods.contains(&"purge"),
        "expected a purge call (rotation): {calls:?}"
    );
    assert!(
        methods.contains(&"copy_to"),
        "expected a copy_to call (new replica): {calls:?}"
    );

    // Ordering: list before purge before copy_to.
    let pos = |m: &str| methods.iter().position(|x| *x == m).unwrap();
    assert!(pos("list") < pos("purge"), "list must precede purge");
    assert!(pos("purge") < pos("copy_to"), "purge must precede copy_to");

    Ok(())
}

/// With no existing replicas and room to spare, no purge should happen; the new
/// replica is still copied.
#[test]
fn remote_backup_no_rotation_when_under_limit() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("data.txt");
    std::fs::write(&src, b"backup me")?;

    let runner = Arc::new(MockRcloneRunner::new().with_list(vec![]));

    let hook = BackupHook {
        exec: HookExecType::Push,
        types: vec![BackupType::Remote],
        local_path: None,
        remote_path: Some("backups/data".to_string()),
        replicas: 3,
    };

    let ctx = HookContext::new(
        src,
        deps_with(Arc::clone(&runner)),
        &mock_remote(),
        &mock_path("remote/data.txt"),
    );

    hook.process(ctx, &AppConfig::default())?;

    let calls = runner.recorded();
    let methods: Vec<&str> = calls.iter().map(|c| c[0].as_str()).collect();

    assert!(methods.contains(&"list"), "expected a list call: {calls:?}");
    assert!(
        !methods.contains(&"purge"),
        "no purge expected under the replica limit: {calls:?}"
    );
    assert!(
        methods.contains(&"copy_to"),
        "expected a copy_to call: {calls:?}"
    );

    Ok(())
}
