use std::net::Ipv4Addr;

use anyhow::Result;

pub trait Dns {
    fn register(&self, ipv4: &Ipv4Addr, domain: &String) -> Result<()>;
}
