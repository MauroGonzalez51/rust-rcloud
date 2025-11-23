use anyhow::Context;
use rcloud::{
    Hook, HookContext, HookContextMetadata, PathConfig, PathConfigHooks, Remote, ZipHook,
    ZipHookConfig,
};
use std::fs;

fn mock_remote() -> Remote {
    Remote {
        id: String::new(),
        remote_name: String::from("drive"),
        provider: String::from("drive"),
    }
}

fn mock_path() -> PathConfig {
    PathConfig {
        id: String::new(),
        remote_id: String::new(),
        local_path: String::new(),
        remote_path: String::new(),
        hash: None,
        tags: vec![],
        hooks: PathConfigHooks {
            push: vec![],
            pull: vec![],
        },
    }
}

#[test]
fn test_zip_single_file() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp_dir")?;

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Hello, World!").context("Failed to write in temp file")?;

    let config = ZipHookConfig {
        exec: rcloud::HookExecType::Push,
        level: Some(6),
        exclude: None,
    };

    let hook = ZipHook::from(config);
    let ctx = HookContext::new(test_file, "", &mock_remote(), &mock_path());
    let result = hook.process(ctx).context("failed to process file")?;

    assert!(result.path.exists());
    assert!(
        result
            .metadata
            .contains_key(&HookContextMetadata::ZipChecksum)
    );

    Ok(())
}

#[test]
fn test_zip_directory() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp_dir")?;

    fs::write(temp_dir.path().join("file1.txt"), b"Content 1")
        .context("failed to write content to file")?;
    fs::write(temp_dir.path().join("file2.txt"), b"Content 2")
        .context("failed to write content to file")?;

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).context("failed to create subdir")?;
    fs::write(subdir.join("file3.txt"), b"Content 3").context("failed to write content to file")?;

    let config = ZipHookConfig {
        exec: rcloud::HookExecType::Push,
        level: Some(6),
        exclude: None,
    };

    let hook = ZipHook::from(config);
    let ctx = HookContext::new(
        temp_dir.path().to_path_buf(),
        "",
        &mock_remote(),
        &mock_path(),
    );
    let result = hook.process(ctx).context("failed to process directory")?;

    assert!(result.path.exists());

    Ok(())
}

#[test]
fn test_zip_with_exclusions() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp dir")?;

    fs::write(temp_dir.path().join("file.txt"), b"Keep")
        .context("failed to write content to file")?;
    fs::write(temp_dir.path().join("file.log"), b"Exclude")
        .context("failed to write content to file")?;

    let config = ZipHookConfig {
        exec: rcloud::HookExecType::Push,
        level: Some(6),
        exclude: Some(vec!["*.log".to_string()]),
    };

    let hook = ZipHook::from(config);
    let ctx = HookContext::new(
        temp_dir.path().to_path_buf(),
        "",
        &mock_remote(),
        &mock_path(),
    );
    let result = hook.process(ctx).context("failed to process file")?;

    assert!(result.path.exists());

    Ok(())
}
