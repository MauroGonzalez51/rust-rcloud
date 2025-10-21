mod config;

use config::prelude::*;

use dotenvy::dotenv;

fn main() -> std::io::Result<()> {
    dotenv().ok();

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let config_path =
        std::env::var("RUST_RCLOUD_CONFIG").expect("[ CRIT ] RUST_RCLOUD_CONFIG is not set");

    let registry = match Registry::load(&config_path) {
        Ok(value) => value,
        Err(err) => match err {
            RegistryError::Io(err) => return Err(err),
            RegistryError::Serde(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("config parse error: {}", err),
                ));
            }
            RegistryError::Custom(err) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
            }
        },
    };

    Ok(())
}
