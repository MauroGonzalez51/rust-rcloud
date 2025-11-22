use crate::hooks::backup::backup_hook::BackupHookReplica;
use anyhow::Context;

pub fn parse_replica(
    path: &std::path::Path,
    re: &regex::Regex,
) -> anyhow::Result<BackupHookReplica> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("failed to get file name")?;

    let caps = re.captures(file_name).context("failed to get captures")?;

    let timestamp = caps
        .get(1)
        .map(|n| n.as_str())
        .context("failed to get timestamp")?
        .parse::<u64>()
        .context("failed to parse timestamp")?;

    let replica_number = caps
        .get(2)
        .map(|n| n.as_str())
        .context("failed to get replica number")?
        .parse::<u32>()
        .context("failed to parse replica number")?;

    Ok(BackupHookReplica {
        path: path.to_path_buf(),
        timestamp,
        replica_number,
    })
}
