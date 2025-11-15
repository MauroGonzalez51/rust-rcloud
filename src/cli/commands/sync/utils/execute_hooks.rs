use crate::config::prelude::{Hook, HookConfig, HookContext};

/// Executes a sequence of hooks over a given hook context, returning the final context.
///
/// # Parameters
/// - `context`: The initial `HookContext` to process.
/// - `hooks`: Slice of `HookConfig` representing the hooks to apply.
///
/// # Returns
/// An `anyhow::Result<HookContext>` containing the processed context after all hooks have been applied.
///
/// # Example
/// ```rust,ignore
/// let context = HookContext::new(some_path);
/// let hooks = vec![HookConfig::Zip(/* ... */)];
/// let result = execute_hooks(context, &hooks)?;
/// ``
pub fn execute_hooks(
    mut context: HookContext,
    hooks: &[HookConfig],
) -> anyhow::Result<HookContext> {
    for hook in hooks {
        let hook: Box<dyn Hook> = Box::from(hook.clone());
        context = hook.process(context)?;
    }

    Ok(context)
}
