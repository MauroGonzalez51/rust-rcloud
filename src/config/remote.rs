use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
    pub id: String,
    pub remote_name: String,
    pub provider: String,
}

impl Remote {
    pub fn show(&self, max: Option<usize>) {
        println!(
            "|{}| {:<width$} ({})",
            self.id,
            self.remote_name,
            self.provider,
            width = max.unwrap_or(0)
        );
    }

    pub fn max_length_name(remotes: &[Remote]) -> usize {
        remotes
            .iter()
            .map(|remote| remote.remote_name.len())
            .max()
            .unwrap_or(20)
    }
}
