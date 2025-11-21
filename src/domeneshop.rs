use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs};

use anyhow::{anyhow, Context, Result};
use netrc_rs::{Machine, Netrc};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::dns_provider::Dns;

const USER_AGENT: &str = "bootdns-domeneshop/0.1";
const REQ_TIMEOUT: Duration = Duration::from_secs(30);
const API_ROOT: &str = "https://api.domeneshop.no/v0/";
const MACHINE_NAME: &str = "api.domeneshop.no";

#[derive(Debug)]
struct Credentials {
    token: String,
    secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Domain {
    domain: String,
    id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsRecord {
    #[serde(skip_serializing)]
    id: i64,

    host: String,

    #[serde(rename = "type")]
    record_type: String,

    data: String,
}

#[derive(Debug)]
pub struct DomeneShop {
    credentials: Credentials,

    client: Client,
    api_root: Url,
    domain2id: HashMap<String, i32>,
    id2domain: HashMap<i32, String>,
}

impl DomeneShop {
    pub(crate) fn new() -> Self {
        let api_root = API_ROOT.parse().unwrap();
        Self {
            credentials: load_netrc().unwrap(),
            client: Client::builder()
                .https_only(true)
                .timeout(Some(REQ_TIMEOUT))
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            api_root,
            domain2id: HashMap::new(),
            id2domain: HashMap::new(),
        }
    }

    fn make_request(&self, method: Method, url: Url) -> RequestBuilder {
        self.client
            .request(method, url)
            .basic_auth(&self.credentials.token, Some(&self.credentials.secret))
    }

    fn get_domain_id(&mut self, domain: &str) -> Result<i32> {
        if self.domain2id.is_empty() {
            let url = self.api_root.join("domains")?;
            let resp = self.make_request(Method::GET, url).send()?;
            let data: Vec<Domain> = resp.json()?;
            for domain_obj in data {
                self.domain2id
                    .insert(domain_obj.domain.clone(), domain_obj.id);
                self.id2domain
                    .insert(domain_obj.id, domain_obj.domain.clone());
            }
        }
        for entry in &self.domain2id {
            if domain.ends_with(entry.0.as_str()) {
                return Ok(*entry.1);
            }
        }
        Err(anyhow!("No registrar domain found that matches {}", domain))
    }

    fn make_host<'a>(&'a self, fqdn: &'a String, domain_id: &i32) -> Result<&'a str> {
        self.id2domain
            .get(domain_id)
            .and_then(|domain| fqdn.strip_suffix(format!(".{}", domain).as_str()))
            .context(anyhow!("Failed to strip domain from {:?}", fqdn))
    }

    fn get_dns_records(&self, fqdn: &String, domain_id: i32) -> Result<Vec<DnsRecord>> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self
            .api_root
            .join(format!("domains/{}/dns", domain_id).as_str())?;
        let resp = self
            .make_request(Method::GET, url)
            .query(&[("host", host)])
            .send()?;
        resp.json().map_err(|e| anyhow!(e))
    }

    fn add_dns_record(&self, fqdn: &String, ipv4: &Ipv4Addr, domain_id: i32) -> Result<Response> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self
            .api_root
            .join(format!("domains/{}/dns", domain_id).as_str())?;
        let dns_record = DnsRecord {
            id: 0,
            host: host.to_string(),
            record_type: "A".to_string(),
            data: ipv4.to_string(),
        };
        self.make_request(Method::POST, url)
            .json(&dns_record)
            .send()
            .map_err(|e| anyhow!(e))
            .inspect(|_resp| {
                info!("Created DNS record {:?}", dns_record);
            })
    }

    fn update_dns_record(
        &self,
        fqdn: &String,
        ipv4: &Ipv4Addr,
        domain_id: i32,
        record_id: i64,
    ) -> Result<Response> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self
            .api_root
            .join(format!("domains/{}/dns/{}", domain_id, record_id).as_str())?;
        let dns_record = DnsRecord {
            id: record_id,
            host: host.to_string(),
            record_type: "A".to_string(),
            data: ipv4.to_string(),
        };
        self.make_request(Method::PUT, url)
            .json(&dns_record)
            .send()
            .map_err(|e| anyhow!(e))
            .inspect(|_resp| {
                info!("Updated DNS record {:?}", dns_record);
            })
    }
}

impl Dns for DomeneShop {
    fn register(&mut self, ipv4: &Ipv4Addr, domain: &str) -> Result<()> {
        debug!(
            "Attempting to register IP {:?} in domain {:?}",
            ipv4, domain
        );
        let hostname = hostname::get()
            .map_err(|_| anyhow!("Unable to get hostname"))?
            .into_string()
            .map_err(|_| anyhow!("Unable to convert hostname to regular string"))?;
        let fqdn = format!("{}.{}", hostname, domain);
        let domain_id = self.get_domain_id(domain)?;
        let dns_record = self.get_dns_records(&fqdn, domain_id);
        match dns_record {
            Ok(records) => {
                if records.is_empty() {
                    self.add_dns_record(&fqdn, ipv4, domain_id)?;
                } else {
                    for record in records {
                        if record.data != ipv4.to_string() {
                            self.update_dns_record(&fqdn, ipv4, domain_id, record.id)?;
                        } else {
                            info!("{} is already up to date", &fqdn);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Get DNS record returned error: {:?}", e);
            }
        }
        Ok(())
    }
}

fn load_netrc() -> Result<Credentials> {
    let netrc_file = env::var("NETRC")
        .map(PathBuf::from)
        .or_else(|_| match dirs::home_dir() {
            Some(home) => Ok(home.join(".netrc")),
            None => Err(anyhow!("Unable to locate home directory")),
        })
        .context("Failed to locate .netrc")?;
    let contents =
        fs::read_to_string(netrc_file).context("Failed to read contents of .netrc file")?;
    let netrc = Netrc::parse(contents, false)
        .map_err(|e| anyhow!(e))
        .context("Failed to parse .netrc file")?;
    let machine = find_machine(&netrc)?;
    let token = match &machine.login {
        Some(token) => Ok(token.clone()),
        None => Err(anyhow!(
            "Token not defined for {:?} in .netrc",
            MACHINE_NAME
        )),
    }?;
    let secret = match &machine.password {
        Some(secret) => Ok(secret.clone()),
        None => Err(anyhow!(
            "Secret not defined for {:?} in .netrc",
            MACHINE_NAME
        )),
    }?;
    Ok(Credentials { token, secret })
}

fn find_machine(netrc: &Netrc) -> Result<&Machine> {
    netrc
        .machines
        .iter()
        .filter(|machine| matches!(&machine.name, Some(name) if name == MACHINE_NAME))
        .next_back()
        .ok_or(anyhow!(
            "Unable to find credentials for {:?} in .netrc file",
            MACHINE_NAME
        ))
}
