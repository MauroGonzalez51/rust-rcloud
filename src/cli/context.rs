use crate::{
    cli::parser::GlobalParameters,
    config::prelude::{AppConfig, Registry},
};

#[derive(Debug, Clone)]
pub struct CommandContext<L = ()> {
    pub global: GlobalParameters,
    pub config: AppConfig,
    pub registry: Registry,
    pub local: L,
}

impl<L: Clone> CommandContext<L> {
    #[allow(dead_code)]
    pub fn new(config: AppConfig, global: GlobalParameters, registry: Registry, local: L) -> Self {
        Self {
            config,
            global,
            registry,
            local,
        }
    }

    pub fn with_args<T: Clone>(&self, args: T) -> CommandContext<T> {
        CommandContext {
            config: self.config.clone(),
            global: self.global.clone(),
            registry: self.registry.clone(),
            local: args.clone(),
        }
    }
}

impl<L: Clone> From<CommandContext<L>> for Registry {
    fn from(context: CommandContext<L>) -> Self {
        context.registry
    }
}

impl From<(AppConfig, GlobalParameters, Registry)> for CommandContext<()> {
    fn from((config, global, registry): (AppConfig, GlobalParameters, Registry)) -> Self {
        Self {
            config,
            global,
            registry,
            local: (),
        }
    }
}

impl<L: Clone> From<(AppConfig, GlobalParameters, Registry, L)> for CommandContext<L> {
    fn from((config, global, registry, local): (AppConfig, GlobalParameters, Registry, L)) -> Self {
        Self {
            config,
            global,
            registry,
            local,
        }
    }
}
