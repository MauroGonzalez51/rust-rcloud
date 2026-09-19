//! Filesystem operations shared by the push and pull sync paths.
//!
//! These extract the noisy, path-shuffling steps out of `push`/`pull` and name
//! them. They are intentionally *not* unified into a single move helper: the
//! push and pull moves have different semantics (rename-in-place with overwrite
//! vs. restore-contents into a destination), so each keeps its own function.

use anyhow::Context;
use std::path::{Path, PathBuf};

/// Locates the file or directory produced by an rclone download inside
/// `temp_dir`.
///
/// If `filename` is given and a matching entry exists, that path is used.
/// Otherwise the temp directory is inspected: a lone entry is returned
/// directly, and anything else falls back to the temp directory itself (the
/// hooks decide what to do with a multi-entry directory).
pub fn find_downloaded(temp_dir: &Path, filename: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(filename) = filename {
        let candidate = temp_dir.join(filename);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    let entries: Vec<_> = std::fs::read_dir(temp_dir)
        .context("failed to read temp directory")?
        .filter_map(Result::ok)
        .collect();

    Ok(match entries.len() {
        1 => entries[0].path(),
        _ => temp_dir.to_path_buf(),
    })
}

/// Renames `from` to `to` (typically a sibling path with the hook-adjusted
/// name), replacing any existing target.
///
/// Used on push to give the processed output its final remote filename. Tries a
/// cheap `std::fs::rename` first and falls back to `fs_extra` when that fails
/// (e.g. across mount points), for both files and directories.
pub fn rename_within(from: &Path, to: &Path) -> anyhow::Result<()> {
    if to.exists() {
        match to.is_dir() {
            true => std::fs::remove_dir_all(to)
                .context("failed to clean up existing temp directory")?,
            false => {
                std::fs::remove_file(to).context("failed to clean up existing temp file")?
            }
        }
    }

    if from.is_file() && std::fs::rename(from, to).is_err() {
        fs_extra::file::move_file(from, to, &fs_extra::file::CopyOptions::new().overwrite(true))
            .context("failed to move file")?;
    }

    if from.is_dir() && std::fs::rename(from, to).is_err() {
        fs_extra::dir::move_dir(from, to, &fs_extra::dir::CopyOptions::new().overwrite(true))
            .with_context(|| {
                format!(
                    "failed to move directory {} to {}",
                    from.display(),
                    to.display()
                )
            })?;
    }

    Ok(())
}

/// Restores the processed content at `from` into the final `dest` path.
///
/// Used on pull to place the downloaded-and-processed content at the local
/// path. A file is moved in place; a directory has its *contents* copied into
/// `dest` (not nested under it) and the source directory is then removed.
pub fn restore_to(from: &Path, dest: &str) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(dest).parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }

    if from.is_file() {
        fs_extra::file::move_file(from, dest, &fs_extra::file::CopyOptions::new())
            .with_context(|| format!("failed to move file to {}", dest))?;
    }

    if from.is_dir() {
        if !Path::new(dest).exists() {
            std::fs::create_dir_all(dest).context("failed to create destination directory")?;
        }

        fs_extra::dir::copy(
            from,
            dest,
            &fs_extra::dir::CopyOptions::new()
                .overwrite(true)
                .content_only(true),
        )
        .with_context(|| format!("failed to move directory to {}", dest))?;

        std::fs::remove_dir_all(from).context("failed to remove temp directory")?;
    }

    Ok(())
}
