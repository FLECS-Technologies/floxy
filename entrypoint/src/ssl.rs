use crate::{error, info};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

const NGINX_SSL_DIR: &str = "/etc/nginx/certs";

pub fn create_nginx_dir() {
    let p = Path::new(NGINX_SSL_DIR);
    if let Err(e) = fs::create_dir_all(p) {
        panic!("Could not create SSL directory {:?}: {}", p, e);
    }
    info!("Created SSL directory {:?}", p);
}

pub fn create_certificates() {
    let key_path = Path::new(NGINX_SSL_DIR).join("key.pem");
    let cert_path = Path::new(NGINX_SSL_DIR).join("cert.pem");

    if key_path.exists() && cert_path.exists() {
        info!("Both key and cert exist");
        return;
    }

    const C: &str = "DE";
    const ST: &str = "Bayern";
    const L: &str = "Kempten (Allgäu)";
    const O: &str = "FLECS Technologies GmbH";
    const CN: &str = "flecs-floxy.local";
    let dn = format!("/C={C}/ST={ST}/L={L}/O={O}/CN={CN}");

    info!("Creating new SSL certificate");
    let mut openssl_cmd = Command::new("openssl");

    openssl_cmd
        .env("OPENSSL_CONF", "/dev/null")
        .args(["req", "-x509"])
        .args(["-newkey", "ec"])
        .args(["-pkeyopt", "ec_paramgen_curve:P-256"])
        .args(["-keyout", key_path.to_str().unwrap()])
        .args(["-out", cert_path.to_str().unwrap()])
        .arg("-nodes")
        .arg("-sha256")
        .args(["-days", "3650"])
        .args(["-subj", dn.as_str()])
        .args(["-addext", "basicConstraints=critical,CA:FALSE"])
        .args([
            "-addext",
            "keyUsage=critical,digitalSignature,keyEncipherment",
        ])
        .args(["-addext", "extendedKeyUsage=serverAuth,clientAuth"])
        .stdin(Stdio::null());

    match openssl_cmd.output() {
        Ok(o) => {
            if !o.status.success() {
                error!(
                    "--- stdout: {}",
                    std::str::from_utf8(&o.stdout).unwrap_or("Invalid output")
                );
                error!(
                    "--- stderr: {}",
                    std::str::from_utf8(&o.stderr).unwrap_or("Invalid output")
                );
                panic!("Could not create SSL certificates:");
            }
        }
        Err(e) => {
            panic!("Could not execute openssl: {e}");
        }
    }
}
