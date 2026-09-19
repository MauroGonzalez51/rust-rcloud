use crate::{
    config::prelude::Remote,
    hooks::{
        backup::{backup_hook::BackupHookReplica, utils},
        prelude::RcloneRunner,
    },
    log_debug,
};
use anyhow::Context;

pub fn get_remote_replicas(
    remote_path: &str,
    rclone: &dyn RcloneRunner,
    remote_info: &Remote,
) -> anyhow::Result<Vec<BackupHookReplica>> {
    let remote_path = std::path::Path::new(remote_path);

    let remote_target = format!(
        "{}:{}",
        remote_info.remote_name,
        remote_path
            .to_str()
            .with_context(|| format!("failed to convert {:?} to str", remote_path))?
    );

    let lines = rclone
        .list(&remote_target)
        .context("failed to execute rclone ls")?;

    let re = regex::Regex::new(r"^(\d+)\.(\d+)$").context("failed to create regex")?;
    let mut replicas = Vec::new();

    for filename in lines {
        let path = std::path::Path::new(&filename);

        if let Ok(replica_info) = utils::parse_replica(path, &re) {
            replicas.push(replica_info);
        }
    }

    log_debug!("remote replicas found: {:?}", replicas);

    Ok(replicas)
}
