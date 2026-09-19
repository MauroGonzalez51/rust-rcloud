use crate::cli::{context::CommandContext, output::OutputSink};

pub fn path_list<S: OutputSink>(context: CommandContext, sink: &S) -> anyhow::Result<()> {
    if context.with_registry()?.paths.is_empty() {
        sink.warn("no paths configured");
        return Ok(());
    }

    for (i, path) in context.with_registry()?.paths.iter().enumerate() {
        let tags_display = match path.tags.is_empty() {
            true => String::new(),
            false => format!(" [tags: {}]", path.tags.join(", ")),
        };

        sink.plain(format!(
            "> {}. {} -> {} [{} Hook(s)]{}",
            i + 1,
            path.local_path,
            path.remote_path,
            path.hooks.pull.len() + path.hooks.push.len(),
            tags_display,
        ));
    }

    Ok(())
}
