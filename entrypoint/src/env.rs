use std::net::Ipv4Addr;

use serde::Deserialize;

pub fn default_webapp_ipv4(gateway: &Ipv4Addr) -> Ipv4Addr {
    Ipv4Addr::from_octets([gateway.octets()[0], gateway.octets()[1], 255, 254])
}

#[derive(Debug, Deserialize)]
pub struct FloxyEnvironment {
    pub http_port: Option<u16>,          // http port floxy listens on
    pub https_port: Option<u16>,         // https port floxy listens on
    pub webapp_ipv4: Option<Ipv4Addr>,   // IPv4 address of container `flecs-webapp`
    pub webapp_http_port: Option<u16>,   // `flecs-webapp` http port
    pub webapp_https_port: Option<u16>,  // `flecs-webapp` https port
    pub flecs_gateway: Option<Ipv4Addr>, // IPv4 address of `flecs-flecsd`
    pub flecs_http_port: Option<u16>,    // Port of `flecs-flecsd`
}
