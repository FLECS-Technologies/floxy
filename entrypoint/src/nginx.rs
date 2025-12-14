use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::Path,
    process::{Child, Command},
    thread::sleep,
    time,
};

use askama::Template;

use crate::{
    env::{FloxyEnvironment, default_webapp_ipv4},
    error, info,
    inotify::Event,
    warn,
};

#[derive(Template)]
#[template(path = "floxy.conf.jinja2")]
struct FloxyConfTemplate {
    floxy_version: String,
    floxy_http_port: u16,
    floxy_https_port: u16,
    webapp_ipv4: Ipv4Addr,
    webapp_http_port: u16,
    webapp_https_port: Option<u16>,
    flecs_gateway: Ipv4Addr,
}

impl From<FloxyEnvironment> for FloxyConfTemplate {
    fn from(value: FloxyEnvironment) -> Self {
        const DEFAULT_HTTP_PORT: u16 = 80;
        const DEFAULT_HTTPS_PORT: u16 = 443;
        const DEFAULT_DOCKER_GATEWAY: [u8; 4] = [172, 21, 0, 1];

        let gateway = value.flecs_gateway.unwrap_or(DEFAULT_DOCKER_GATEWAY.into());
        Self {
            floxy_version: format!(
                "{}-{}",
                env!("CARGO_PKG_VERSION"),
                option_env!("GIT_SHA")
                    .filter(|v| !v.is_empty())
                    .unwrap_or("unknown")
            ),
            floxy_http_port: value.http_port.unwrap_or(DEFAULT_HTTP_PORT),
            floxy_https_port: value.https_port.unwrap_or(DEFAULT_HTTPS_PORT),
            webapp_ipv4: value.webapp_ipv4.unwrap_or(default_webapp_ipv4(&gateway)),
            webapp_http_port: value.webapp_http_port.unwrap_or(DEFAULT_HTTP_PORT),
            webapp_https_port: value.webapp_https_port,
            flecs_gateway: gateway,
        }
    }
}

const NGINX_STARTUP_POLL_INTERVAL_MS: u64 = 125;
const NGINX_STARTUP_TIMEOUT_MS: u64 = 30000;

const FLOXY_CONFIG_PATH: &str = "/etc/nginx/conf.d/floxy.conf";

fn validate_config(config: &FloxyConfTemplate) {
    if config.floxy_http_port == config.floxy_https_port {
        panic!("Invalid configuration: floxy_http_port == floxy_https_port");
    }

    // Get a list of all IP addresses referring to localhost
    let mut addrs = Vec::new();
    for iface in nix::ifaddrs::getifaddrs().expect("getifaddrs should always succeed") {
        if let Some(addr) = iface.address {
            if let Some(ip) = addr.as_sockaddr_in() {
                addrs.push(IpAddr::V4(ip.ip()));
            } else if let Some(ip) = addr.as_sockaddr_in6() {
                addrs.push(IpAddr::V6(ip.ip()));
            }
        }
    }

    // Check if webapp_ipv4 points to a local IP address
    if addrs.contains(&config.webapp_ipv4.into()) {
        // If so, make sure floxy and webapp do not share ports
        if config.floxy_http_port == config.webapp_http_port {
            panic!("Invalid configuration: floxy_http_port == webapp_http_port");
        }
        if config
            .webapp_https_port
            .as_ref()
            .is_some_and(|port| port == &config.floxy_https_port)
        {
            panic!("Invalid configuration: floxy_https_port == webapp_https_port");
        }
    }
}

pub(crate) fn create_config() {
    let env = match envy::prefixed("FLOXY_").from_env::<FloxyEnvironment>() {
        Ok(env) => {
            info!("Using environment {env:?}");
            env
        }
        Err(e) => panic!("Failed to parse environment variables: {e:?}"),
    };

    let conf_template = FloxyConfTemplate::from(env);
    validate_config(&conf_template);

    fs::create_dir_all(Path::new(FLOXY_CONFIG_PATH).parent().unwrap())
        .expect("Could not create config directory");
    info!("Created config directory");
    fs::write(FLOXY_CONFIG_PATH, conf_template.to_string()).expect("Could not write floxy.conf");
    info!("Wrote nginx config");
}

pub(crate) fn spawn() -> Child {
    info!("Spawning nginx");
    let child = Command::new("nginx")
        .args([
            "-c",
            "/etc/nginx/nginx.conf",
            "-e",
            "/dev/stderr",
            "-g",
            "daemon off;",
        ])
        .spawn();
    if let Err(e) = child {
        panic!("Failed to execute nginx: {e}");
    }

    /* Wait for pidfile creation -> nginx is up and running */
    let pid_path = Path::new("/run/nginx.pid");
    let start_time = std::time::Instant::now();

    info!("Waiting for nginx...");
    while !pid_path.is_file() {
        assert!(
            start_time.elapsed().as_millis() <= NGINX_STARTUP_TIMEOUT_MS.into(),
            "nginx startup timeout"
        );
        sleep(time::Duration::from_millis(NGINX_STARTUP_POLL_INTERVAL_MS));
    }
    info!("nginx ready");

    child.unwrap()
}

pub(crate) fn check_reload(ev: Event) {
    info!("Validating new configuration due to {ev:?}");
    let Ok(result) = Command::new("nginx")
        .args(["-c", "/etc/nginx/nginx.conf", "-e", "/dev/stderr", "-t"])
        .spawn()
        .and_then(|mut p| p.wait())
    else {
        error!("Could not execute nginx to test configuration");
        return;
    };
    if !result.success() {
        warn!("nginx -t exited with non-zero exit-code {result}");
        return;
    }

    let Ok(result) = Command::new("nginx")
        .args([
            "-c",
            "/etc/nginx/nginx.conf",
            "-e",
            "/dev/stderr",
            "-s",
            "reload",
        ])
        .spawn()
        .and_then(|mut p| p.wait())
    else {
        error!("Could not execute nginx to reload configuration");
        return;
    };
    if !result.success() {
        error!("nginx -s exited with non-zero exit-code {result}");
    }
}
