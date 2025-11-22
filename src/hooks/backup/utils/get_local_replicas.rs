use crate::hooks::backup::{backup_hook::BackupHookReplica, utils};
use anyhow::Context;

pub fn get_local_replicas(local_path: Option<&str>) -> anyhow::Result<Vec<BackupHookReplica>> {
    let directory = std::path::Path::new(local_path.expect("local path must be declared"));

    let mut replicas = Vec::new();
    let re = regex::Regex::new(r"^(\d+)\.(\d+)$").context("failed to create regex")?;

    if directory.exists() && directory.is_dir() {
        for entry in std::fs::read_dir(directory)
            .with_context(|| format!("failed to read directory: {:?}", directory))?
        {
            let entry = entry.context("failed to get directory entry")?;
            let path = entry.path();

            if let Ok(replica_info) = utils::parse_replica(&path, &re) {
                replicas.push(replica_info)
            }
        }
    }

    Ok(replicas)
}
