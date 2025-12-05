use crate::{
    cli::parser::GlobalParameters,
    config::prelude::{AppConfig, Registry},
};
use std::ops::{Deref, DerefMut};

pub struct CommandContext<L = ()> {
    pub global: GlobalParameters,
    pub config: AppConfig,
    pub registry: Registry,
    pub local: L,
}

impl<L> CommandContext<L> {
    pub fn new(config: AppConfig, global: GlobalParameters, registry: Registry, local: L) -> Self {
        Self {
            config,
            global,
            registry,
            local,
        }
    }
}

impl<L> Deref for CommandContext<L> {
    type Target = Registry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

impl<L> DerefMut for CommandContext<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registry
    }
}

impl<L> From<CommandContext<L>> for Registry {
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

impl<L> From<(AppConfig, GlobalParameters, Registry, L)> for CommandContext<L> {
    fn from((config, global, registry, local): (AppConfig, GlobalParameters, Registry, L)) -> Self {
        Self {
            config,
            global,
            registry,
            local,
        }
    }
}
