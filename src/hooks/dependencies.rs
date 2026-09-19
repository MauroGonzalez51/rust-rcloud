//! Injectable side-effect capabilities threaded through the hook pipeline.
//!
//! Hooks need two impure capabilities to do their work: running the `rclone`
//! binary and acquiring the encryption password. Rather than calling
//! `std::process::Command` and `inquire` directly inside `Hook::process`
//! (which makes hooks impossible to exercise headless), those effects are
//! expressed as the [`RcloneRunner`] and [`PasswordProvider`] traits and
//! injected via [`HookDependencies`] on the [`HookContext`].
//!
//! Production wires up [`ProcessRcloneRunner`] (spawns the real binary) and
//! [`InteractivePasswordProvider`] (prompts on the terminal). Tests substitute
//! mocks that record calls and return canned results, so a full
//! `Hook::process` can run without a terminal or a real `rclone`.

use anyhow::Context;
use std::sync::Arc;

/// Runs `rclone` subcommands on behalf of the pipeline.
///
/// The method set is deliberately semantic (one method per operation the code
/// actually performs) rather than a generic "run these args" call: it keeps
/// every rclone-specific string, flag, and environment variable in one place
/// (the production implementation) instead of scattered across call sites.
pub trait RcloneRunner: Send + Sync {
    /// Transfers `src` to `dst`, inheriting stdout so `rclone`'s live progress
    /// is visible. Uses `copyto` for files and `copy` for directories. The
    /// fixed transfer flags (`--progress --checksum --transfers=8
    /// --checkers=16`) are applied internally.
    fn transfer(&self, src: &str, dst: &str) -> anyhow::Result<std::process::ExitStatus>;

    /// Copies `src` to `dst` silently (captured, no progress). Returns
    /// `Ok(true)` when the copy succeeded and `Ok(false)` when the source did
    /// not exist (rclone exited non-zero). Used for remote backup replicas.
    fn copy_to(&self, src: &str, dst: &str) -> anyhow::Result<bool>;

    /// Lists entries under `remote_path` via `lsf`, one entry per returned
    /// line. Returns an empty vector when the path does not exist. The caller
    /// parses or displays the lines.
    fn list(&self, remote_path: &str) -> anyhow::Result<Vec<String>>;

    /// Deletes `remote_path` via `purge` (with the drive trash disabled). A
    /// failed deletion is logged, not fatal; only a spawn failure returns an
    /// error.
    fn purge(&self, remote_path: &str) -> anyhow::Result<()>;
}

/// Acquires the encryption password.
///
/// This only *obtains* the secret (a terminal prompt in production, a fixed
/// value in tests). Verifying it against the stored Argon2 hash is a separate,
/// pure step owned by the encryption hook, so this trait needs no knowledge of
/// the hash.
pub trait PasswordProvider: Send + Sync {
    /// Returns the password to encrypt/decrypt with.
    fn get(&self) -> anyhow::Result<String>;
}

/// The impure capabilities a hook pipeline needs, injected via the
/// [`HookContext`](crate::hooks::prelude::HookContext).
///
/// Both fields are always present: the providers are cheap to construct and do
/// nothing until invoked, so there is no need to make the password optional for
/// pipelines that happen not to encrypt.
#[derive(Clone)]
pub struct HookDependencies {
    /// Runs `rclone` subcommands.
    pub rclone: Arc<dyn RcloneRunner>,
    /// Acquires the encryption password.
    pub password: Arc<dyn PasswordProvider>,
}

impl std::fmt::Debug for HookDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookDependencies")
            .field("rclone", &"<dyn RcloneRunner>")
            .field("password", &"<dyn PasswordProvider>")
            .finish()
    }
}

impl HookDependencies {
    /// Builds the production dependency set: a [`ProcessRcloneRunner`] spawning
    /// the binary at `rclone_path` and an [`InteractivePasswordProvider`].
    pub fn production(rclone_path: impl Into<String>) -> Self {
        Self {
            rclone: Arc::new(ProcessRcloneRunner::new(rclone_path)),
            password: Arc::new(InteractivePasswordProvider),
        }
    }
}

// --- Production implementations --------------------------------------------

/// Fixed transfer flags applied to every [`RcloneRunner::transfer`] call.
const TRANSFER_FLAGS: &[&str] = &["--progress", "--checksum", "--transfers=8", "--checkers=16"];

/// Runs `rclone` by spawning the real binary at `path`.
#[derive(Clone)]
pub struct ProcessRcloneRunner {
    path: String,
}

impl ProcessRcloneRunner {
    /// Creates a runner that spawns the `rclone` binary at `path`.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl RcloneRunner for ProcessRcloneRunner {
    fn transfer(&self, src: &str, dst: &str) -> anyhow::Result<std::process::ExitStatus> {
        // `copyto` preserves the destination name for a single file; `copy`
        // mirrors a directory's contents.
        let verb = match std::path::Path::new(src).is_file() {
            true => "copyto",
            false => "copy",
        };

        let mut args = vec![verb, src, dst];
        args.extend_from_slice(TRANSFER_FLAGS);

        std::process::Command::new(&self.path)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .status()
            .context("failed to spawn rclone process")
    }

    fn copy_to(&self, src: &str, dst: &str) -> anyhow::Result<bool> {
        let output = std::process::Command::new(&self.path)
            .args(["copyto", src, dst])
            .output()
            .context("failed to execute rclone copyto")?;

        Ok(output.status.success())
    }

    fn list(&self, remote_path: &str) -> anyhow::Result<Vec<String>> {
        let output = std::process::Command::new(&self.path)
            .args(["lsf", remote_path])
            .output()
            .context("failed to execute rclone lsf")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    }

    fn purge(&self, remote_path: &str) -> anyhow::Result<()> {
        let output = std::process::Command::new(&self.path)
            .args(["purge", remote_path])
            .env("RCLONE_DRIVE_USE_TRASH", "false")
            .output()
            .with_context(|| format!("failed to purge remote path: {}", remote_path))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            crate::log_warn!("failed to purge remote path: {} ({})", remote_path, stderr);
        }

        Ok(())
    }
}

/// Prompts for the encryption password on the terminal.
#[derive(Clone, Default)]
pub struct InteractivePasswordProvider;

impl PasswordProvider for InteractivePasswordProvider {
    fn get(&self) -> anyhow::Result<String> {
        inquire::Password::new("Enter encryption password:")
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()
            .context("failed to read password input")
    }
}
