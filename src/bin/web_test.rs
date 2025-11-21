#[macro_use]
extern crate log;

use std::time::Duration;

use anyhow::{anyhow, Result};
use log::LevelFilter;
use reqwest::blocking::Client;
use url::Url;

const USER_AGENT: &str = "bootdns-domeneshop/0.1";
const REQ_TIMEOUT: Duration = Duration::from_secs(30);
const URL: &str = "https://httpbin.org/json";

fn main() -> Result<()> {
    env_logger::builder()
        .default_format()
        .filter_level(LevelFilter::Debug)
        .init();

    let client = Client::builder()
        .https_only(true)
        .timeout(Some(REQ_TIMEOUT))
        .user_agent(USER_AGENT)
        .build()?;

    let url: Url = URL.parse()?;

    let resp = client.get(url).send().map_err(|e| anyhow!(e))?;
    debug!("Response: {:?}", resp);

    Ok(())
}
