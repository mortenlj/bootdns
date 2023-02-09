#[macro_use]
extern crate log;

use std::cmp::min;

use anyhow::{anyhow, Context, Result};
use env_logger::Env;


fn main() -> Result<()> {
    init_logging();

    Ok(())
}

/// Configure logging taking verbosity into account
fn init_logging() {
    let log_levels = vec!["error", "warning", "info", "debug"];
    let default_level = 1;
    let env = Env::default().filter_or("LOG_LEVEL", log_levels[default_level]);
    env_logger::init_from_env(env);
}
