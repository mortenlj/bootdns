use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Duration;

use anyhow::{anyhow, Result, Context};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder, Request, Response};
use url::Url;

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

    agent: Agent,
    api_root: Url,
    domain2id: HashMap<String, i32>,
    id2domain: HashMap<i32, String>,
}

impl DomeneShop {
    pub(crate) fn new(token: String, secret: String) -> Result<impl Dns> {
        let api_root = API_ROOT.parse()?;
        Ok(Self {
            credentials: Credentials {
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
            api_root,
            domain2id: HashMap::new(),
            id2domain: HashMap::new(),
        })
    }

    fn make_request(&self, method: &str, url: &Url) -> Request {
        self.agent.request_url(method, url)
            .set("Authorization", format!("Basic {}", self.credentials.as_header()).as_str())
    }

    fn get_domain_id(&mut self, domain: &String) -> Result<i32> {
        if self.domain2id.is_empty() {
            let url = self.api_root.join("domains")?;
            let resp = self.make_request("GET", &url).call()?;
            let data: Vec<Domain> = resp.into_json()?;
            for domain_obj in data {
                self.domain2id.insert(domain_obj.domain.clone(), domain_obj.id);
                self.id2domain.insert(domain_obj.id, domain_obj.domain.clone());
            }
        }
        for entry in &self.domain2id {
            if domain.ends_with(entry.0.as_str()) {
                return Ok(*entry.1);
            }
        }
        Err(anyhow!("No registrar domain found that matches {}", domain))
    }

    fn make_host<'a>(&'a self, fqdn: &'a String, domain_id: &i32) -> Result<&str> {
        self.id2domain.get(&domain_id)
            .and_then(|domain| fqdn.strip_suffix(format!(".{}", domain).as_str()))
            .context(anyhow!("Failed to strip domain from {:?}", fqdn))
    }

    fn get_dns_records(&self, fqdn: &String, domain_id: i32) -> Result<Vec<DnsRecord>> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self.api_root.join(format!("domains/{}/dns", domain_id).as_str())?;
        let resp = self.make_request("GET", &url).query("host", host).call()?;
        resp.into_json().map_err(|e| anyhow!(e))
    }

    fn add_dns_record(&self, fqdn: &String, ipv4: &Ipv4Addr, domain_id: i32) -> Result<Response> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self.api_root.join(format!("domains/{}/dns", domain_id).as_str())?;
        let dns_record = DnsRecord {
            id: 0,
            host: host.to_string(),
            record_type: "A".to_string(),
            data: ipv4.to_string(),
        };
        self.make_request("POST", &url)
            .send_json(&dns_record)
            .map_err(|e| anyhow!(e))
            .and_then(|resp| {
                info!("Created DNS record {:?}", dns_record);
                Ok(resp)
            })
    }

    fn update_dns_record(&self, fqdn: &String, ipv4: &Ipv4Addr, domain_id: i32, record_id: i64) -> Result<Response> {
        let host = self.make_host(fqdn, &domain_id)?;
        let url = self.api_root.join(format!("domains/{}/dns/{}", domain_id, record_id).as_str())?;
        let dns_record = DnsRecord {
            id: record_id,
            host: host.to_string(),
            record_type: "A".to_string(),
            data: ipv4.to_string(),
        };
        self.make_request("PUT", &url)
            .send_json(&dns_record)
            .map_err(|e| anyhow!(e))
            .and_then(|resp| {
                info!("Updated DNS record {:?}", dns_record);
                Ok(resp)
            })
    }
}

impl Dns for DomeneShop {
    fn register(&mut self, ipv4: &Ipv4Addr, domain: &String) -> Result<()> {
        let hostname = hostname::get().map_err(|_| anyhow!("Unable to get hostname"))?
            .into_string().map_err(|_| anyhow!("Unable convert hostname to regular string"))?;
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
                        }
                    }
                }
            }
            Err(e) => {
                info!("Get DNS record returned error: {:?}", e);
            }
        }
        Ok(())
    }
}


