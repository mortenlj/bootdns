use std::net::Ipv4Addr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};
use crate::dns_provider::Dns;


const USER_AGENT: &'static str = "bootdns-domeneshop/0.1";
const IO_TIMEOUT: u64 = 10;
const REQ_TIMEOUT: u64 = 30;


#[derive(Debug)]
struct Credentials {
    token: String,
    secret: String,
}


#[derive(Debug)]
pub struct DomeneShop {
    credentials: Credentials,

    agent: Agent,
}

impl DomeneShop {
    pub(crate) fn new(token: String, secret: String) -> impl Dns {
        Self {
            credentials: Credentials{
                token,
                secret,
            },
            agent: AgentBuilder::new()
                .https_only(true)
                .timeout_read(Duration::from_secs(IO_TIMEOUT))
                .timeout_write(Duration::from_secs(IO_TIMEOUT))
                .timeout(Duration::from_secs(REQ_TIMEOUT))
                .user_agent(USER_AGENT)
                .build(),
        }
    }
}

impl Dns for DomeneShop {
    fn register(&self, ipv4: &Ipv4Addr, domain: &String) -> Result<()> {
        let hostname = hostname::get().map_err(|_| anyhow!("Unable to get hostname"))?
            .into_string().map_err(|_| anyhow!("Unable convert hostname to regular string"))?;
        let fqdn = format!("{}.{}", hostname, domain);
        info!("TODO: Create DNS record {} {}", fqdn, &ipv4);
        Ok(())
    }
}

