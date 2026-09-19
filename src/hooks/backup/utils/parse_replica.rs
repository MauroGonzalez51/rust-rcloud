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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn re() -> regex::Regex {
        regex::Regex::new(r"^(\d+)\.(\d+)$").unwrap()
    }

    #[test]
    fn parses_timestamp_and_replica_number() {
        let replica = parse_replica(Path::new("/backups/1700000000.3"), &re()).unwrap();
        assert_eq!(replica.timestamp, 1_700_000_000);
        assert_eq!(replica.replica_number, 3);
        assert_eq!(replica.path, Path::new("/backups/1700000000.3"));
    }

    #[test]
    fn rejects_non_matching_name() {
        assert!(parse_replica(Path::new("/backups/not-a-replica"), &re()).is_err());
        assert!(parse_replica(Path::new("/backups/123"), &re()).is_err());
        assert!(parse_replica(Path::new("/backups/1700000000.3.txt"), &re()).is_err());
    }

    #[test]
    fn overflowing_timestamp_errors() {
        // Larger than u64::MAX -> parse failure.
        let name = "99999999999999999999999999.1";
        assert!(parse_replica(Path::new(name), &re()).is_err());
    }
}
