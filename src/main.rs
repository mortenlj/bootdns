#[macro_use]
extern crate log;

use std::net::IpAddr;
use std::str::FromStr;
use anyhow::{Context, Result};
use cidr::Ipv4Cidr;

use figment::Figment;
use figment::providers::{Env, Serialized};
use if_addrs;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    cidrs: Vec<Ipv4Cidr>,
    log_level: String,
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::default("log_level", "error"))
        .merge(Env::prefixed("BOOTDNS_"))
        .extract()?;

    init_logging(config.log_level)?;

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

/// Configure logging
fn init_logging(log_level: String) -> Result<()> {
    let level_filter: LevelFilter = LevelFilter::from_str(&log_level)
        .context("failed to create LevelFilter from log level string")?;
    env_logger::builder()
        .default_format()
        .filter_level(level_filter)
        .init();
    Ok(())
}
