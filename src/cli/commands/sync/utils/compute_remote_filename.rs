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
    if hooks.is_empty() {
        return base_name.to_string();
    }

    let last = hooks
        .iter()
        .filter(|hook| hook.modifies_filename())
        .next_back();

    match last {
        Some(hook) => match hook.hook_type() {
            Hooks::Zip => format!("{}.zip", base_name),
            _ => base_name.to_string(),
        },
        None => base_name.to_string(),
    }
}
