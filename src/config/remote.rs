use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
    pub id: String,
    pub remote_name: String,
    pub provider: String,
}
