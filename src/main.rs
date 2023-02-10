#[macro_use]
extern crate log;

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
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
struct Credentials {
    token: String,
    secret: String,
}

#[derive(Deserialize, Serialize)]
struct Config {
    domain_maps: Vec<DomainMap>,
    log_level: String,
    credentials: Credentials,
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::default("log_level", "error"))
        .merge(Env::prefixed("BOOTDNS_").split("__"))
        .extract()?;

    init_logging(config.log_level)?;

    let dns_provider = DomeneShop::new(config.credentials.token, config.credentials.secret)?;

    for iface in if_addrs::get_if_addrs().unwrap() {
        if let IpAddr::V4(ipv4) = iface.addr.ip() {
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
