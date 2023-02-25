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

use crate::dns_provider::Dns;
use crate::domeneshop::DomeneShop;

mod domeneshop;
mod dns_provider;

#[derive(Deserialize, Serialize)]
struct DomainMap {
    cidr: Ipv4Cidr,
    domain: String,
}

#[derive(Deserialize, Serialize)]
struct Config {
    domain_maps: Vec<DomainMap>,
    log_level: String,
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::default("log_level", "error"))
        .merge(Env::prefixed("BOOTDNS_").split("__"))
        .extract()?;

    init_logging(config.log_level)?;
    debug!("Logging initialized ...");

    let mut dns_provider = DomeneShop::new()?;
    debug!("DNS provider ready ...");

    for iface in if_addrs::get_if_addrs().unwrap() {
        debug!("Evaluating interface {:?}", iface);
        if let IpAddr::V4(ipv4) = iface.addr.ip() {
            debug!("Checking IP {:?} against domain maps", ipv4);
            for dm in &config.domain_maps {
                if dm.cidr.contains(&ipv4) {
                    dns_provider.register(&ipv4, &dm.domain)?;
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
