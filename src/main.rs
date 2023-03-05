#[macro_use]
extern crate log;

use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use cidr::Ipv4Cidr;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml, Yaml};
use if_addrs;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

use crate::dns_provider::Dns;
use crate::domeneshop::DomeneShop;

mod domeneshop;
mod dns_provider;

#[derive(Deserialize, Serialize, Debug)]
struct DomainMap {
    cidr: Ipv4Cidr,
    domain: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    domain_maps: Vec<DomainMap>,
    log_level: String,
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::default("log_level", "info"))
        .merge(Toml::file(locate_file("toml")))
        .merge(Yaml::file(locate_file("yaml")))
        .merge(Env::prefixed("BOOTDNS_").split("__"))
        .extract()?;

    init_logging(&config.log_level)?;
    debug!("Logging initialized ...");
    debug!("Configuration: {:#?}", &config);

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
fn init_logging(log_level: &String) -> Result<()> {
    let level_filter: LevelFilter = LevelFilter::from_str(log_level)
        .context("failed to create LevelFilter from log level string")?;
    env_logger::builder()
        .default_format()
        .filter_level(level_filter)
        .init();
    Ok(())
}

/// Locate config file
fn locate_file(format: &str) -> PathBuf {
    let mut locations: Vec<PathBuf> = Vec::new();

    match env::var("BOOTDNS_CONFIG_FILE")
        .map(|var| PathBuf::from(var).with_extension(format.clone()))
        .map_err(|e| anyhow!(e)) {
        Ok(filepath) => locations.push(filepath),
        _ => {}
    }

    match dirs::config_dir() {
        Some(config) => locations.push(config.join("bootdns").with_extension(format.clone())),
        None => {}
    };

    match dirs::home_dir() {
        Some(home) => locations.push(home.join("bootdns").with_extension(format.clone())),
        None => {}
    };

    for filepath in locations {
        if filepath.is_file() {
            return filepath
        }
    }

    PathBuf::from("bootdns").with_extension(format)
}
