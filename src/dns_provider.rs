use std::net::Ipv4Addr;

use anyhow::Result;

pub trait Dns {
    fn register(&mut self, ipv4: &Ipv4Addr, domain: &str) -> Result<()>;
}
