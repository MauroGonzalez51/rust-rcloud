use anyhow::Context;

pub fn expand_path(path: &str) -> anyhow::Result<std::path::PathBuf> {
    std::fs::canonicalize(shellexpand::tilde(path).to_string())
        .with_context(|| format!("failed to expand path: {:?}", path))
}
