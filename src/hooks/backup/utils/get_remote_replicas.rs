use crate::{
    config::prelude::Remote,
    hooks::backup::{backup_hook::BackupHookReplica, utils},
    log_debug,
};
use anyhow::Context;

pub fn get_remote_replicas(
    remote_path: &str,
    rclone_path: &str,
    remote_info: &Remote,
) -> anyhow::Result<Vec<BackupHookReplica>> {
    let remote_path = std::path::Path::new(remote_path);

    let output = std::process::Command::new(rclone_path)
        .args([
            "lsf",
            &format!(
                "{}:{}",
                remote_info.remote_name,
                remote_path
                    .to_str()
                    .with_context(|| format!("failed to convert {:?} to str", remote_path))?
            ),
        ])
        .output()
        .context("failed to execute rclone ls")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let re = regex::Regex::new(r"^(\d+)\.(\d+)$").context("failed to create regex")?;
    let mut replicas = Vec::new();

    log_debug!("remote replicas found: {:?}", replicas);

    for filename in String::from_utf8_lossy(&output.stdout).lines() {
        let path = std::path::Path::new(filename);

        if let Ok(replica_info) = utils::parse_replica(path, &re) {
            replicas.push(replica_info);
        }
    }

    Ok(replicas)
}
