use nix::{sys::signal::Signal, unistd::Pid};

use crate::signal_handler::SHUTDOWN_SIGNAL;

mod env;
mod inotify;
mod logging;
mod nginx;
mod signal_handler;
mod ssl;

fn main() {
    /* Basic nginx setup */
    ssl::create_nginx_dir();
    ssl::create_certificates();
    nginx::create_config();

    /* Initialize signal handling; store old_mask for inotify */
    let old_mask = signal_handler::init();

    /* Initialize inotify watches for configuration changes */
    let mut ctx = inotify::init(old_mask);

    /* Start nginx in background and wait for it */
    let mut nginx_process = nginx::spawn();

    /* Process events until shutdown is requested */
    while SHUTDOWN_SIGNAL.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        inotify::process_events(&mut ctx);
    }

    let signum = SHUTDOWN_SIGNAL.load(std::sync::atomic::Ordering::Relaxed);

    println!("Shutting down nginx with signal {signum}");
    let _ = nix::sys::signal::kill(
        Pid::from_raw(nginx_process.id() as i32),
        Signal::try_from(signum).unwrap(),
    );

    let _ = nginx_process.wait();

    info!("Goodbye");
}
