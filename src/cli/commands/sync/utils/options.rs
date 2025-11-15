use crate::{
    config::prelude::{HookExecType, PathConfig},
    log_info, log_warn,
};
use anyhow::Context;

/// Cleans the local path before synchronization if the direction is Pull and the `clean` flag is set.
///
/// # Parameters
/// - `direction`: The synchronization direction (`Push` or `Pull`).
/// - `clean`: Whether to clean the local path before syncing.
/// - `local_path`: The local path to clean.
///
/// # Returns
/// An `anyhow::Result<()>` indicating success or failure.
///
/// # Behavior
/// - If direction is `Push`, cleaning is ignored.
/// - If direction is `Pull` and `clean` is true, removes the local path if it exists.
pub fn clean(direction: &HookExecType, clean: &bool, local_path: &str) -> anyhow::Result<()> {
    match direction {
        HookExecType::Push => {
            if *clean {
                log_info!(
                    "current direction is {}. clean will be ignored",
                    HookExecType::Push
                );
            }

            Ok(())
        }
        HookExecType::Pull => {
            if *clean && std::path::Path::new(local_path).exists() {
                std::fs::remove_dir_all(local_path)
                    .with_context(|| format!("failed to clean target directory: {}", local_path))?
            }

            Ok(())
        }
    }
}

/// Determines whether synchronization should proceed based on hash comparison and force flag.
///
/// # Parameters
/// - `direction`: The synchronization direction (`Push` or `Pull`).
/// - `force`: Whether to force synchronization regardless of hash.
/// - `path_config`: The path configuration containing the stored hash and local path.
/// - `processed_hash`: The newly computed hash of the local content.
///
/// # Returns
/// An `anyhow::Result<bool>`.  
/// - Returns `Ok(true)` if synchronization should continue.
/// - Returns `Ok(false)` if synchronization should be skipped (hashes match and not forced).
///
/// # Behavior
/// - If `force` is false and the stored hash matches the processed hash, synchronization is skipped.
/// - For `Pull`, also checks if the local path exists before skipping.
pub fn force(
    direction: &HookExecType,
    force: &bool,
    path_config: &PathConfig,
    processed_hash: &str,
) -> anyhow::Result<bool> {
    match direction {
        HookExecType::Push => {
            if !force {
                if let Some(stored_hash) = &path_config.hash {
                    if stored_hash == processed_hash {
                        log_warn!("content unchanged (hash match). skipping");
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        }
        HookExecType::Pull => {
            if !force {
                if let Some(stored_hash) = &path_config.hash {
                    let local_path_exists = std::path::Path::new(&path_config.local_path).exists();

                    if stored_hash == processed_hash && local_path_exists {
                        log_warn!("content unchanged (hash match). skipping");
                        return Ok(false);
                    }

                    if !local_path_exists {
                        log_info!("local path does not exists, syncing despite hash match");
                    }
                }
            }

            Ok(true)
        }
    }
}
