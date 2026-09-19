//! Tests for remote filename computation.
//!
//! `compute_remote_filename` decides the name a payload takes on the remote
//! after filename-modifying hooks (Zip -> `.zip`, Encryption -> `.enc`) run.
//! Push uses it to name the upload; pull uses it to locate the download. If the
//! two disagree, pull "can't find" the file — the bug we most want to pin down.

use rcloud::HookExecType;
use rcloud::cli::commands::sync::utils::{compute_remote_filename, resolve_remote_filename};
use rcloud::{BackupHookConfig, BackupType, EncryptionHookConfig, HookConfig, ZipHookConfig};

fn zip(exec: HookExecType) -> HookConfig {
    HookConfig::Zip(ZipHookConfig {
        exec,
        level: Some(6),
        exclude: None,
    })
}

fn encryption(exec: HookExecType) -> HookConfig {
    HookConfig::Encryption(EncryptionHookConfig {
        exec,
        hash: String::new(),
    })
}

fn backup(exec: HookExecType) -> HookConfig {
    HookConfig::Backup(BackupHookConfig {
        exec,
        types: vec![BackupType::Local],
        local_path: None,
        remote_path: None,
        replicas: 1,
    })
}

#[test]
fn no_hooks_keeps_base_name() {
    assert_eq!(compute_remote_filename(&[], "archive"), "archive");
}

#[test]
fn zip_appends_zip() {
    let hooks = [zip(HookExecType::Push)];
    assert_eq!(compute_remote_filename(&hooks, "archive"), "archive.zip");
}

#[test]
fn encryption_appends_enc() {
    let hooks = [encryption(HookExecType::Push)];
    assert_eq!(compute_remote_filename(&hooks, "archive"), "archive.enc");
}

#[test]
fn backup_does_not_change_name() {
    let hooks = [backup(HookExecType::Push)];
    assert_eq!(compute_remote_filename(&hooks, "archive"), "archive");
}

#[test]
fn zip_then_encryption_stacks_extensions() {
    let hooks = [zip(HookExecType::Push), encryption(HookExecType::Push)];
    assert_eq!(
        compute_remote_filename(&hooks, "archive"),
        "archive.zip.enc"
    );
}

#[test]
fn backup_between_filename_hooks_is_ignored() {
    let hooks = [
        zip(HookExecType::Push),
        backup(HookExecType::Push),
        encryption(HookExecType::Push),
    ];
    assert_eq!(
        compute_remote_filename(&hooks, "archive"),
        "archive.zip.enc"
    );
}

/// The push side is the single source of truth for the remote filename.
///
/// Push computes the name from its hook order and records it in
/// `PathConfig.remote_filename`. Pull must reuse that stored value rather than
/// recomputing from its own (reverse-ordered) hook list, so the two always
/// agree regardless of how the pull pipeline is ordered.
#[test]
fn pull_reuses_pushed_remote_filename() {
    let push_hooks = [zip(HookExecType::Push), encryption(HookExecType::Push)];

    // Push records this on the PathConfig.
    let stored = compute_remote_filename(&push_hooks, "archive");
    assert_eq!(stored, "archive.zip.enc");

    // Pull must use the stored value verbatim — not recompute from pull hooks,
    // which would yield the wrong "archive.enc.zip".
    let pull_hooks = [encryption(HookExecType::Pull), zip(HookExecType::Pull)];
    let recomputed = compute_remote_filename(&pull_hooks, "archive");
    assert_ne!(
        stored, recomputed,
        "guard: recomputing from pull hooks is known to disagree; \
         pull must rely on the stored name instead"
    );
}

// --- resolve_remote_filename: the option-3 fix, testable without rclone ---

#[test]
fn resolve_prefers_stored_filename() {
    // Even with pull hooks in reverse order, the stored name wins verbatim.
    let pull_hooks = [encryption(HookExecType::Pull), zip(HookExecType::Pull)];
    let resolved = resolve_remote_filename(Some("archive.zip.enc"), &pull_hooks, "backups/archive");
    assert_eq!(resolved.as_deref(), Some("archive.zip.enc"));
}

#[test]
fn resolve_falls_back_to_computed_when_not_stored() {
    // Legacy path (never pushed with tracking): compute from hooks + remote_path.
    let hooks = [zip(HookExecType::Pull), encryption(HookExecType::Pull)];
    let resolved = resolve_remote_filename(None, &hooks, "backups/archive");
    assert_eq!(resolved.as_deref(), Some("archive.zip.enc"));
}

#[test]
fn resolve_none_when_no_stored_and_no_name_hooks() {
    let hooks = [backup(HookExecType::Pull)];
    let resolved = resolve_remote_filename(None, &hooks, "backups/archive");
    assert_eq!(resolved, None);
}

#[test]
fn resolve_stored_wins_even_over_no_hooks() {
    let resolved = resolve_remote_filename(Some("archive.zip"), &[], "backups/archive");
    assert_eq!(resolved.as_deref(), Some("archive.zip"));
}
