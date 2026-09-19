//! Mirror of `src/utils/path.rs`.
//!
//! `expand_path` expands a leading `~` and canonicalizes the result, so the
//! target must exist on disk.

use rcloud::utils::expand_path;
use std::fs;

#[test]
fn expands_existing_absolute_path() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("real.txt");
    fs::write(&file, b"x")?;

    let expanded = expand_path(file.to_str().unwrap())?;
    // Canonicalized form points at the same file.
    assert_eq!(expanded, fs::canonicalize(&file)?);
    Ok(())
}

#[test]
fn expands_tilde_to_home() -> anyhow::Result<()> {
    // `~` should resolve to the home directory (which exists), so canonicalize
    // succeeds and the result starts at home.
    if let Some(home) = dirs_home() {
        let expanded = expand_path("~")?;
        assert_eq!(expanded, fs::canonicalize(&home)?);
    }
    Ok(())
}

#[test]
fn missing_path_errors() {
    assert!(expand_path("/nonexistent/rcloud/test/path/xyz").is_err());
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
