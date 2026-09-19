//! Shared test doubles for the injectable hook dependencies.
//!
//! These let integration tests drive a full `Hook::process` (including the
//! encryption and backup hooks, which in production prompt for a password and
//! shell out to `rclone`) without a terminal or a real `rclone` binary.

use rcloud::{HookDependencies, PasswordProvider, RcloneRunner};
use std::sync::{Arc, Mutex};

/// Records every rclone call and returns canned results.
#[derive(Default)]
pub struct MockRcloneRunner {
    /// Invocations as `(method, args...)`, in call order.
    pub calls: Mutex<Vec<Vec<String>>>,
    /// Lines returned by `list`.
    pub list_result: Vec<String>,
    /// Value returned by `copy_to`.
    pub copy_to_result: bool,
}

impl MockRcloneRunner {
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            list_result: Vec::new(),
            copy_to_result: true,
        }
    }

    pub fn with_list(mut self, lines: Vec<String>) -> Self {
        self.list_result = lines;
        self
    }

    pub fn with_copy_to(mut self, ok: bool) -> Self {
        self.copy_to_result = ok;
        self
    }

    fn record(&self, parts: &[&str]) {
        self.calls
            .lock()
            .unwrap()
            .push(parts.iter().map(|s| s.to_string()).collect());
    }

    /// Snapshot of recorded calls.
    pub fn recorded(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }
}

impl RcloneRunner for MockRcloneRunner {
    fn transfer(&self, src: &str, dst: &str) -> anyhow::Result<std::process::ExitStatus> {
        self.record(&["transfer", src, dst]);
        Ok(success_status())
    }

    fn copy_to(&self, src: &str, dst: &str) -> anyhow::Result<bool> {
        self.record(&["copy_to", src, dst]);
        Ok(self.copy_to_result)
    }

    fn list(&self, remote_path: &str) -> anyhow::Result<Vec<String>> {
        self.record(&["list", remote_path]);
        Ok(self.list_result.clone())
    }

    fn purge(&self, remote_path: &str) -> anyhow::Result<()> {
        self.record(&["purge", remote_path]);
        Ok(())
    }
}

/// Returns a fixed password without prompting.
pub struct MockPasswordProvider {
    pub password: String,
}

impl MockPasswordProvider {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
        }
    }
}

impl PasswordProvider for MockPasswordProvider {
    fn get(&self) -> anyhow::Result<String> {
        Ok(self.password.clone())
    }
}

/// Builds a [`HookDependencies`] backed by the given mocks.
pub fn test_dependencies(
    rclone: Arc<dyn RcloneRunner>,
    password: Arc<dyn PasswordProvider>,
) -> HookDependencies {
    HookDependencies { rclone, password }
}

/// Convenience: dependencies with no-op mocks (empty password, always-ok rclone).
pub fn default_test_dependencies() -> HookDependencies {
    test_dependencies(
        Arc::new(MockRcloneRunner::new()),
        Arc::new(MockPasswordProvider::new("")),
    )
}

/// A successful `ExitStatus` portable across platforms.
fn success_status() -> std::process::ExitStatus {
    // Spawning `true`/cmd is the simplest portable way to obtain a real
    // successful ExitStatus without platform-specific from_raw.
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
    #[cfg(not(unix))]
    {
        std::process::Command::new("cmd")
            .args(["/C", "exit 0"])
            .status()
            .expect("failed to produce success status")
    }
}
