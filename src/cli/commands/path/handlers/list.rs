use console::Style;

use crate::{cli::parser::Args, config::prelude::*, log_warn};

pub fn path_list(_args: &Args, registry: &Registry) {
    if registry.paths.is_empty() {
        log_warn!("no paths configured");
        return;
    }

    let idx_style = Style::new().bold().cyan();
    let local_style = Style::new().green();
    let arrow_style = Style::new().bold().yellow();
    let remote_style = Style::new().blue();
    let hooks_style = Style::new().bold().magenta();

    for (i, path) in registry.paths.iter().enumerate() {
        println!(
            "{} {} {} {} {} {}",
            idx_style.apply_to(format!("> {}.", i + 1)),
            local_style.apply_to(&path.local_path),
            arrow_style.apply_to("->"),
            remote_style.apply_to(&path.remote_path),
            hooks_style.apply_to(format!(
                "[{} Hook(s)]",
                path.hooks.pull.len() + path.hooks.push.len()
            )),
            Style::new().dim().apply_to(""),
        );
    }
}
