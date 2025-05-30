use crate::config::schema::{Config, Remote};
use uuid::Uuid;

pub struct Action;

impl Action {
    pub fn list(config: &Config) {
        let max = Remote::max_length_name(&config.remotes);

        for remote in config.remotes.iter() {
            remote.show(Some(max));
        }
    }

    pub fn add(config: &mut Config, name: String, provider: String) {
        config.remotes.push(Remote {
            id: Uuid::new_v4().to_string(),
            remote_name: name,
            provider,
        });
        config.save();
    }

    pub fn remove(config: &mut Config, id: String) {
        config.remotes.retain(|remote| remote.id != id);
        config.save()
    }

    pub fn update(
        config: &mut Config,
        id: String,
        new_name: Option<String>,
        new_provider: Option<String>,
    ) {
        if let Some(remote) = config.remotes.iter_mut().find(|remote| remote.id == id) {
            if let Some(name) = new_name {
                remote.remote_name = name;
            }

            if let Some(provider) = new_provider {
                remote.provider = provider
            }

            config.save();
            return;
        }

        println!("[ ERROR ] remote with id |{}| not found", id);
    }

    pub fn find(config: &Config, id: Option<String>, name: Option<String>, or: bool) {
        if or {
            let remotes: Vec<Remote> = config
                .remotes
                .iter()
                .filter(|remote| {
                    id.as_ref().is_some_and(|id| remote.id == *id)
                        || name
                            .as_ref()
                            .is_some_and(|name| remote.remote_name.contains(name))
                })
                .cloned()
                .collect();

            let max = Remote::max_length_name(&remotes);

            for remote in remotes {
                remote.show(Some(max));
            }
        }

        let remotes: Vec<Remote> = config
            .remotes
            .iter()
            .filter(|remote| {
                id.as_ref().is_none_or(|id| remote.id == *id)
                    && name
                        .as_ref()
                        .is_none_or(|name| remote.remote_name.contains(name))
            })
            .cloned()
            .collect();

        let max = Remote::max_length_name(&remotes);

        for remote in remotes {
            remote.show(Some(max));
        }
    }
}
