use crate::{
    config::prelude::{HookConfig, HookExecType},
    log_warn,
};

pub fn check_hooks(hooks: &[HookConfig], target_exec: &HookExecType) {
    for hook in hooks {
        if hook.exec_type() != target_exec {
            log_warn!(
                "hook {} has incorrect exec type. Found: {}, Expected: {}",
                hook.hook_type(),
                hook.exec_type(),
                target_exec
            );
        }
    }
}
