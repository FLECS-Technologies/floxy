use std::sync::atomic::AtomicI32;

use nix::{
    libc,
    sys::signal::{
        SaFlags, SigAction, SigHandler, SigSet, SigmaskHow,
        Signal::{SIGINT, SIGQUIT, SIGTERM},
        sigaction, sigprocmask,
    },
};

pub static SHUTDOWN_SIGNAL: AtomicI32 = AtomicI32::new(0);

extern "C" fn signal_handler(signum: libc::c_int) {
    match signum {
        libc::SIGINT | libc::SIGTERM | libc::SIGQUIT => {
            SHUTDOWN_SIGNAL.store(signum, std::sync::atomic::Ordering::Relaxed);
        }
        _ => {}
    }
}

pub fn init() -> SigSet {
    /* Setup signal handling */
    let sa = SigAction::new(
        SigHandler::Handler(signal_handler),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        sigaction(SIGINT, &sa).expect("Setting signal handlers should always succeed");
        sigaction(SIGTERM, &sa).expect("Setting signal handlers should always succeed");
        sigaction(SIGQUIT, &sa).expect("Setting signal handlers should always succeed");
    }

    let mut block_mask = SigSet::empty();
    block_mask.add(SIGINT);
    block_mask.add(SIGTERM);
    block_mask.add(SIGQUIT);

    let mut old_mask = SigSet::empty();
    sigprocmask(
        SigmaskHow::SIG_BLOCK,
        Some(&block_mask),
        Some(&mut old_mask),
    )
    .expect("Changing the signal mask should always succeed");

    old_mask
}
