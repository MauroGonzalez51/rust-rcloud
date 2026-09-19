//! Mirror of `src/config/app.rs`.
//!
//! Exercises `AppConfig::load` (default creation from the embedded template,
//! parsing a custom file, rejecting invalid TOML) and the `Default`-derived
//! key bindings.

use rcloud::AppConfig;
use std::fs;

#[test]
fn load_creates_default_when_missing() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rcloud.toml");

    let cfg = AppConfig::load(&path)?;

    // The file is materialized from the embedded default on first load.
    assert!(path.exists());
    // Default key bindings are the vim-style ones.
    assert_eq!(cfg.tui.keys.quit, vec!['q']);
    assert_eq!(cfg.tui.keys.up, vec!['k']);
    assert_eq!(cfg.tui.keys.down, vec!['j']);
    assert_eq!(cfg.tui.keys.left, vec!['h']);
    assert_eq!(cfg.tui.keys.right, vec!['l']);
    Ok(())
}

#[test]
fn load_parses_custom_temp_path() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rcloud.toml");
    fs::write(
        &path,
        r#"
[core]
temp_path = "/tmp/rcloud-custom"
"#,
    )?;

    let cfg = AppConfig::load(&path)?;
    assert_eq!(
        cfg.core.temp_path.as_deref(),
        Some(std::path::Path::new("/tmp/rcloud-custom"))
    );
    Ok(())
}

#[test]
fn load_parses_custom_keybindings() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rcloud.toml");
    fs::write(
        &path,
        r#"
[core]

[tui.keys]
quit = ['x']
up = ['w']
down = ['s']
left = ['a']
right = ['d']
"#,
    )?;

    let cfg = AppConfig::load(&path)?;
    assert_eq!(cfg.tui.keys.quit, vec!['x']);
    assert_eq!(cfg.tui.keys.right, vec!['d']);
    Ok(())
}

#[test]
fn load_missing_tui_section_falls_back_to_defaults() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rcloud.toml");
    fs::write(&path, "[core]\n")?;

    let cfg = AppConfig::load(&path)?;
    // tui defaults in when omitted.
    assert_eq!(cfg.tui.keys.quit, vec!['q']);
    assert!(cfg.core.temp_path.is_none());
    Ok(())
}

#[test]
fn load_invalid_toml_errors() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rcloud.toml");
    fs::write(&path, "this is = not valid = toml [[[")?;

    assert!(AppConfig::load(&path).is_err());
    Ok(())
}

#[test]
fn default_has_vim_keybindings() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.tui.keys.quit, vec!['q']);
    assert_eq!(cfg.tui.keys.up, vec!['k']);
}
