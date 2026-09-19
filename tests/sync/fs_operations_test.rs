//! Tests for the shared push/pull filesystem helpers.

use rcloud::cli::commands::sync::utils::{find_downloaded, rename_within, restore_to};
use std::fs;

// --- find_downloaded -------------------------------------------------------

#[test]
fn find_downloaded_prefers_existing_named_candidate() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let named = dir.path().join("archive.zip");
    fs::write(&named, b"x")?;
    // A second entry ensures we picked the named one, not the lone-entry rule.
    fs::write(dir.path().join("other.txt"), b"y")?;

    let found = find_downloaded(dir.path(), Some("archive.zip"))?;
    assert_eq!(found, named);
    Ok(())
}

#[test]
fn find_downloaded_falls_back_to_lone_entry_when_name_missing() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let only = dir.path().join("downloaded.bin");
    fs::write(&only, b"x")?;

    // Named candidate does not exist -> single entry is used.
    let found = find_downloaded(dir.path(), Some("nope.zip"))?;
    assert_eq!(found, only);
    Ok(())
}

#[test]
fn find_downloaded_returns_dir_when_multiple_entries_and_no_name() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("a.txt"), b"a")?;
    fs::write(dir.path().join("b.txt"), b"b")?;

    let found = find_downloaded(dir.path(), None)?;
    assert_eq!(found, dir.path());
    Ok(())
}

#[test]
fn find_downloaded_single_entry_no_name() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let only = dir.path().join("solo.txt");
    fs::write(&only, b"x")?;

    let found = find_downloaded(dir.path(), None)?;
    assert_eq!(found, only);
    Ok(())
}

// --- rename_within ---------------------------------------------------------

#[test]
fn rename_within_moves_file_to_new_name() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let from = dir.path().join("temp.out");
    let to = dir.path().join("final.zip");
    fs::write(&from, b"payload")?;

    rename_within(&from, &to)?;

    assert!(!from.exists(), "source should be gone");
    assert_eq!(fs::read(&to)?, b"payload");
    Ok(())
}

#[test]
fn rename_within_overwrites_existing_target() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let from = dir.path().join("temp.out");
    let to = dir.path().join("final.zip");
    fs::write(&from, b"new")?;
    fs::write(&to, b"stale")?;

    rename_within(&from, &to)?;

    assert_eq!(fs::read(&to)?, b"new", "target should be replaced");
    Ok(())
}

#[test]
fn rename_within_moves_directory() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let from = dir.path().join("tmpdir");
    fs::create_dir(&from)?;
    fs::write(from.join("inner.txt"), b"content")?;
    let to = dir.path().join("named");

    rename_within(&from, &to)?;

    assert!(!from.exists());
    assert_eq!(fs::read(to.join("inner.txt"))?, b"content");
    Ok(())
}

// --- restore_to ------------------------------------------------------------

#[test]
fn restore_to_moves_file_creating_parent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let from = dir.path().join("processed.bin");
    fs::write(&from, b"restored")?;
    // Destination parent does not exist yet.
    let dest = dir.path().join("nested/deep/out.bin");

    restore_to(&from, dest.to_str().unwrap())?;

    assert!(!from.exists());
    assert_eq!(fs::read(&dest)?, b"restored");
    Ok(())
}

#[test]
fn restore_to_copies_directory_contents_and_removes_source() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let from = dir.path().join("extracted");
    fs::create_dir(&from)?;
    fs::write(from.join("a.txt"), b"alpha")?;
    let sub = from.join("sub");
    fs::create_dir(&sub)?;
    fs::write(sub.join("b.txt"), b"beta")?;

    let dest = dir.path().join("target");

    restore_to(&from, dest.to_str().unwrap())?;

    // Contents land directly under dest (content_only), source is gone.
    assert!(!from.exists(), "source dir should be removed");
    assert_eq!(fs::read(dest.join("a.txt"))?, b"alpha");
    assert_eq!(fs::read(dest.join("sub/b.txt"))?, b"beta");
    Ok(())
}
