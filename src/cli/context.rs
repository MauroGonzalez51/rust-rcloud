use crate::{cli::parser::GlobalParameters, config::prelude::Registry};
use std::ops::{Deref, DerefMut};

pub struct CommandContext<L = ()> {
    pub global: GlobalParameters,
    pub registry: Registry,
    pub local: L,
}

impl<L> CommandContext<L> {
    pub fn new(global: GlobalParameters, registry: Registry, local: L) -> Self {
        Self {
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

impl From<(GlobalParameters, Registry)> for CommandContext<()> {
    fn from((global, registry): (GlobalParameters, Registry)) -> Self {
        Self {
            global,
            registry,
            local: (),
        }
    }
}

impl<L> From<(GlobalParameters, Registry, L)> for CommandContext<L> {
    fn from((global, registry, local): (GlobalParameters, Registry, L)) -> Self {
        Self {
            global,
            registry,
            local,
        }
    }
}
