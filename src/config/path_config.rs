use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfig {
    pub id: String,
    pub remote_id: String,
    pub local_path: String,
    pub remote_path: String,
}
