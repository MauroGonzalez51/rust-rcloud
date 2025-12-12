use crate::{
    cli::parser::GlobalParameters,
    config::prelude::{AppConfig, Registry},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct CommandContext<L = ()> {
    pub global: std::sync::Arc<GlobalParameters>,
    pub config: std::sync::Arc<AppConfig>,
    pub registry: std::sync::Arc<Mutex<Registry>>,
    pub local: L,
}

impl<L: Clone> CommandContext<L> {
    pub fn new(config: AppConfig, global: GlobalParameters, registry: Registry, local: L) -> Self {
        Self {
            config: Arc::new(config),
            global: Arc::new(global),
            registry: Arc::new(Mutex::new(registry)),
            local,
        }
    }

    pub fn with_args<T: Clone>(&self, args: T) -> CommandContext<T> {
        CommandContext {
            config: Arc::clone(&self.config),
            global: Arc::clone(&self.global),
            registry: Arc::clone(&self.registry),
            local: args.clone(),
        }
    }

    pub fn with_registry(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Registry>> {
        self.registry.lock().map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl<L: Clone> From<CommandContext<L>> for Arc<Mutex<Registry>> {
    fn from(context: CommandContext<L>) -> Self {
        context.registry
    }
}

impl From<(AppConfig, GlobalParameters, Registry)> for CommandContext<()> {
    fn from((config, global, registry): (AppConfig, GlobalParameters, Registry)) -> Self {
        Self::new(config, global, registry, ())
    }
}

impl<L: Clone> From<(AppConfig, GlobalParameters, Registry, L)> for CommandContext<L> {
    fn from((config, global, registry, local): (AppConfig, GlobalParameters, Registry, L)) -> Self {
        Self::new(config, global, registry, local)
    }
}

impl From<(Arc<AppConfig>, Arc<GlobalParameters>, Arc<Mutex<Registry>>)> for CommandContext<()> {
    fn from(
        (config, global, registry): (Arc<AppConfig>, Arc<GlobalParameters>, Arc<Mutex<Registry>>),
    ) -> Self {
        Self {
            config,
            global,
            registry,
            local: (),
        }
    }
}

impl<L: Clone>
    From<(
        Arc<AppConfig>,
        Arc<GlobalParameters>,
        Arc<Mutex<Registry>>,
        L,
    )> for CommandContext<L>
{
    fn from(
        (config, global, registry, local): (
            Arc<AppConfig>,
            Arc<GlobalParameters>,
            Arc<Mutex<Registry>>,
            L,
        ),
    ) -> Self {
        Self {
            config,
            global,
            registry,
            local,
        }
    }
}
