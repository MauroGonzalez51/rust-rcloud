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
    let registry = Registry::load(&config_path).expect("[ Error ] could not parse config");

    Ok(())
}
