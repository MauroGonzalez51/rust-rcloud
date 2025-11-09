use crate::config::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfig {
    pub id: String,

    pub remote_id: String,
    pub local_path: String,
    pub remote_path: String,

    #[serde(default)]
    pub tags: Vec<String>,

    pub hooks: PathConfigHooks,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfigHooks {
    #[serde(default)]
    pub push: Vec<HookConfig>,

    #[serde(default)]
    pub pull: Vec<HookConfig>,
}
