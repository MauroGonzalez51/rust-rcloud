use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip)]
    pub config_path: String,
}

impl Config {
    pub fn load(config_path: &str) -> Self {
        if Path::new(&config_path).exists() {
            match serde_json::from_str::<Config>(
                &fs::read_to_string(config_path).expect("[ ERROR ] failed to read config file"),
            ) {
                Ok(mut config) => {
                    config.config_path = config_path.to_string();
                    return config;
                }
                Err(_) => {
                    eprintln!("[ ERROR ] failed to parse config, creating default config instead");

                    let default_config = Config {
                        config_path: config_path.to_string(),
                    };

                    default_config.save();
                    return default_config;
                }
            }
        }

        let default_config = Config {
            config_path: config_path.to_string(),
        };

        default_config.save();

        default_config
    }

    pub fn save(&self) {
        let path = Path::new(&self.config_path);

        let data = serde_json::to_string_pretty(self).expect("Failed to serialize config");

        fs::write(path, data).expect("Failed to write config file");
    }
}
