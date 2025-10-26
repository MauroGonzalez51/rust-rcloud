use crate::config::prelude::Remote;
use crate::{cli::parser::Args, config::prelude::Registry};
use console::Style;
use inquire::Text;
use uuid::Uuid;

pub fn remote_add(
    args: &Args,
    registry: &mut Registry,
    name: &Option<String>,
    provider: &Option<String>,
) {
    let name = match name {
        Some(value) => value,
        None => {
            match Text::new("Provide the remote name:")
                .with_help_message(
                    "Must be the same that you inserted when configuring the remote in 'rcloud'",
                )
                .prompt()
            {
                Ok(value) => &value.clone(),
                Err(err) => {
                    eprintln!("[ ERROR ] failed to read remote name: {err}");
                    return;
                }
            }
        }
    };

    let provider = match provider {
        Some(value) => value,
        None => match Text::new("Provide the Remote Provider:").prompt() {
            Ok(value) => &value.clone(),
            Err(err) => {
                eprintln!("[ ERROR ] failed to read remote provider: {err}");
                return;
            }
        },
    };

    if args.verbose > 0 {
        println!("[ INFO ] adding remote '{name}' ({provider}) to registry")
    }

    match registry.tx(|reg| {
        reg.remotes.push(Remote {
            id: Uuid::new_v4().to_string(),
            remote_name: name.to_string(),
            provider: provider.to_string(),
        });
    }) {
        Ok(_) => {
            println!(
                "{}",
                Style::new()
                    .green()
                    .apply_to("[ INFO ] remote added succesfully")
            );
        }
        Err(err) => eprintln!("{}", err),
    }
}
