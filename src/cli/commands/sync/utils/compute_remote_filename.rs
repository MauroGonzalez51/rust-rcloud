use crate::config::prelude::HookConfig;

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
    if hooks.is_empty() {
        return base_name.to_string();
    }

    let last_hook = &hooks[hooks.len() - 1];

    match last_hook {
        HookConfig::Zip(_) => format!("{}.zip", base_name),
        HookConfig::Backup(_) => todo!(),
    }
}
