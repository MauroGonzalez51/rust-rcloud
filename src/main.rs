mod config;

use dotenvy::dotenv;

fn main() {
    dotenv().ok();

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // let config_path = std::env::var("RUST_RCLOUD_CONFIG").expect("RUST_RCLOUD_CONFIG is not set");

    todo!("redo the whole stuff");
}
