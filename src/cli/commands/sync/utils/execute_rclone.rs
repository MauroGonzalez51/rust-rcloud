use anyhow::Context;

/// Executes an rclone command with the provided arguments.
///
/// # Parameters
/// - `rclone_path`: Path to the rclone executable.
/// - `source_path`: Source path for the copy operation.
/// - `target_path`: Target path for the copy operation.
/// - `args`: Optional slice of additional arguments to pass to rclone.
///
/// # Returns
/// An `anyhow::Result<std::process::ExitStatus>` representing the result of the rclone process execution.
///
/// # Example
/// ```rust,ignore
/// let status = execute_rclone(
///     "rclone",
///     "/path/to/source",
///     "remote:path",
///     Some(&["--dry-run"]),
/// )?;
/// ``
pub fn execute_rclone(
    rclone_path: &str,
    source_path: &str,
    target_path: &str,
    args: Option<&[&str]>,
) -> anyhow::Result<std::process::ExitStatus> {
    let mut cmd_args = vec![
        "copy",
        source_path,
        target_path,
        "--progress",
        "--checksum",
        "--delete-during",
        "--transfers=8",
        "--checkers=16",
    ];

    if let Some(extra) = args {
        cmd_args.extend_from_slice(extra);
    }

    std::process::Command::new(rclone_path)
        .args(cmd_args)
        .status()
        .context("failed to execute rclone")
}
