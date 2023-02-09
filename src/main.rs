#[macro_use]
extern crate log;

use std::net::{IpAddr, Ipv4Addr};
use anyhow::{anyhow, Context, Result};
use cidr::Ipv4Cidr;
use env_logger::Env as LogEnv;

use figment::Figment;
use figment::providers::{Env, Serialized};
use if_addrs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    cidrs: Vec<Ipv4Cidr>,
    log_level: usize,
}

fn main() -> Result<()> {
    let defaults: Config = Config{
        cidrs: vec![],
        log_level: 1,
    };

    let config: Config = Figment::new()
        .merge(Serialized::defaults(defaults))
        .merge(Env::prefixed("BOOTDNS_"))
        .extract()?;

    init_logging(config.log_level);

    for iface in if_addrs::get_if_addrs().unwrap() {
        if let IpAddr::V4(ipv4) = iface.addr.ip() {
            for cidr in &config.cidrs {
                if cidr.contains(&ipv4) {
                    info!("Registering name for {}", &ipv4);
                }
            }
        }
    }

    Ok(())
}

/// Configure logging taking verbosity into account
fn init_logging(default_level: usize) {
    let log_levels = vec!["error", "warning", "info", "debug"];
    let env = LogEnv::default().filter_or("LOG_LEVEL", log_levels[default_level]);
    env_logger::init_from_env(env);
}
