use crate::cli::{context::CommandContext, output::OutputSink};

pub fn remote_list<S: OutputSink>(context: CommandContext, sink: &S) -> anyhow::Result<()> {
    if context.with_registry()?.remotes.is_empty() {
        sink.warn("no remotes were found");
        return Ok(());
    }

    for (i, remote) in context.with_registry()?.remotes.iter().enumerate() {
        sink.plain(format!(
            "> {}. {} ({}) [id: {}]",
            i + 1,
            remote.remote_name,
            remote.provider,
            remote.id
        ));
    }

    Ok(())
}
