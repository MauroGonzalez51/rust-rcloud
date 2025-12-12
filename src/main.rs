mod cli;
mod config;
mod hooks;
mod tui;
mod utils;

use crate::{cli::run, utils::prelude::logger};
use dotenvy::dotenv;

fn main() {
    dotenv().ok();

    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(err) = run::run() {
        logger().with_context(&err);
        std::process::exit(1);
    }
}
