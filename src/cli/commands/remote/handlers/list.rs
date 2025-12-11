use crate::cli::context::CommandContext;
use console::Style;

pub fn remote_list(context: CommandContext) {
    let remote_name = Style::new().bold().green();
    let remote_provider = Style::new().italic();
    let remote_id = Style::new().underlined();

    if context.registry.remotes.is_empty() {
        println!(
            "{}",
            Style::new()
                .bold()
                .yellow()
                .apply_to("[ WARN ] no remotes were found")
        )
    }

    for (i, remote) in context.registry.remotes.iter().enumerate() {
        println!(
            "> {}. {} ({}) [id: {}]",
            i + 1,
            remote_name.apply_to(&remote.remote_name),
            remote_provider.apply_to(&remote.provider),
            remote_id.apply_to(&remote.id)
        )
    }
}
