use std::net::Ipv4Addr;

use serde::Deserialize;

pub fn default_webapp_ipv4(gateway: &Ipv4Addr) -> Ipv4Addr {
    Ipv4Addr::from_octets([gateway.octets()[0], gateway.octets()[1], 255, 254])
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FloxyEnvironment {
    pub http_port: Option<u16>,
    pub https_port: Option<u16>,
    pub webapp_ipv4: Option<Ipv4Addr>,
    pub webapp_http_port: Option<u16>,
    pub webapp_https_port: Option<u16>,
    pub flecs_gateway: Option<Ipv4Addr>,
}
