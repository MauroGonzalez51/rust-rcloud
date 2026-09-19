use crate::config::prelude::{HookConfig, Hooks};

/// Computes the final remote filename based on the applied hooks.
///
/// # Parameters
/// - `hooks`: Slice of hooks to be applied to the file.
/// - `base_name`: The base name of the remote file.
///
/// # Returns
/// A `String` containing the final remote filename, including the extension if required.
///
/// # Example
/// ```rust, ignore
/// let hooks = vec![HookConfig::Zip(Default::default())];
/// let filename = compute_remote_filename(&hooks, "backup");
/// assert_eq!(filename, "backup.zip");
/// ```
pub fn compute_remote_filename(hooks: &[HookConfig], base_name: &str) -> String {
    hooks
        .iter()
        .filter(|hook| hook.modifies_filename())
        .fold(base_name.to_string(), |acc, hook| match hook.hook_type() {
            Hooks::Zip => format!("{}.zip", acc),
            Hooks::Encryption => format!("{}.enc", acc),
            _ => acc,
        })
}

/// Resolves the remote filename to use for a pull.
///
/// Prefers `stored` — the filename recorded by the push that created the remote
/// copy (the single source of truth). Falls back to computing it from `hooks`
/// and `remote_path` only when nothing was stored (a path pushed before the
/// filename was tracked, or an external upload). Returns `None` when no
/// filename-modifying hook applies and nothing was stored, meaning the payload
/// keeps its plain `remote_path`.
pub fn resolve_remote_filename(
    stored: Option<&str>,
    hooks: &[HookConfig],
    remote_path: &str,
) -> Option<String> {
    if let Some(name) = stored {
        return Some(name.to_string());
    }

    if hooks.iter().any(|h| h.modifies_filename()) {
        let base = std::path::Path::new(remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");
        return Some(compute_remote_filename(hooks, base));
    }

    None
}
