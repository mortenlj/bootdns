use std::net::Ipv4Addr;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder, serde_json};
use url::Url;
use base64::{Engine as _, engine::general_purpose};
use crate::dns_provider::Dns;


const USER_AGENT: &'static str = "bootdns-domeneshop/0.1";
const IO_TIMEOUT: u64 = 10;
const REQ_TIMEOUT: u64 = 30;

const API_ROOT: &'static str = "https://api.domeneshop.no/v0/";


#[derive(Debug)]
struct Credentials {
    token: String,
    secret: String,
}

impl Credentials {
    pub(crate) fn as_header(&self) -> String {
        let auth = format!("{}:{}", self.token, self.secret);
        general_purpose::STANDARD.encode(auth)
    }
}


#[derive(Debug)]
pub struct DomeneShop {
    credentials: Credentials,

    agent: Agent,
    api_root: Url,
}

impl DomeneShop {
    pub(crate) fn new(token: String, secret: String) -> Result<impl Dns> {
        let api_root = API_ROOT.parse()?;
        Ok(Self {
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
            api_root
        })
    }
}

impl Dns for DomeneShop {
    fn register(&self, ipv4: &Ipv4Addr, domain: &String) -> Result<()> {
        let hostname = hostname::get().map_err(|_| anyhow!("Unable to get hostname"))?
            .into_string().map_err(|_| anyhow!("Unable convert hostname to regular string"))?;
        let fqdn = format!("{}.{}", hostname, domain);
        let url = self.api_root.join("domains")?;
        let resp = self.agent.request_url("GET", &url)
            .set("Authorization", format!("Basic {}", self.credentials.as_header()).as_str())
            .call();
        let data: serde_json::Value = resp.unwrap().into_json()?;
        debug!("{:?}", data);

        info!("TODO: Create DNS record {} {}", fqdn, &ipv4);
        Ok(())
    }
}

