use std::{
    collections::HashMap,
    fs,
    os::fd::AsFd,
    path::{Path, PathBuf},
};

use nix::{
    poll::{PollFlags, ppoll},
    sys::{
        inotify::{AddWatchFlags, InitFlags, Inotify, WatchDescriptor},
        signal::SigSet,
    },
};

use crate::{error, info, nginx, warn};

const PRIVATE_CONF_PATH: &str = "/etc/nginx/conf.d/floxy";
const SHARED_CONF_PATH: &str = "/tmp/floxy/conf.d";
const FLOXY_CONF_SUBDIRS: [&str; 2] = ["instances", "servers"];

pub(crate) struct Context {
    inotify: Inotify,
    wds: HashMap<WatchDescriptor, PathBuf>,
    ss: SigSet,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Event<'a> {
    MovedTo(&'a Path),
    Delete(&'a Path),
}

impl Context {
    fn new() -> Self {
        Self {
            inotify: Inotify::init(InitFlags::IN_CLOEXEC | InitFlags::IN_NONBLOCK)
                .expect("Inotify instance should always initialize successfully"),
            wds: HashMap::new(),
            ss: SigSet::empty(),
        }
    }

    fn default() -> Self {
        let mut ctx = Context::new();

        for dir in FLOXY_CONF_SUBDIRS {
            let p = Path::new(SHARED_CONF_PATH).join(dir);
            info!("Adding inotify watch for {p:?}");
            ctx.wds.insert(
                ctx.inotify
                    .add_watch(&p, AddWatchFlags::IN_CLOSE_WRITE | AddWatchFlags::IN_DELETE)
                    .unwrap_or_else(|_| panic!("Watching path {p:?} should always succeed")),
                Path::new(dir).to_owned(),
            );
        }
        ctx
    }
}

fn create_conf_subdirs(path: &Path) {
    for dir in FLOXY_CONF_SUBDIRS {
        let dir = path.join(dir);
        fs::create_dir_all(&dir).expect("Creating conf directory should always succeed");
        info!("Created conf directory {dir:?}");
    }
}

fn create_private_conf_dirs() {
    let path = Path::new(PRIVATE_CONF_PATH);
    let _ = fs::remove_dir_all(path);
    create_conf_subdirs(path);
}

fn create_shared_conf_dirs() {
    let path = Path::new(SHARED_CONF_PATH);
    create_conf_subdirs(path);
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    info!("Copying {} to {}", from.display(), to.display());
    let entries = fs::read_dir(from);
    if entries.is_err() {
        return;
    }
    for entry in entries.unwrap() {
        if entry.is_err() {
            warn!("Skipping erroneous entry {entry:?}");
            continue;
        }
        let entry = entry.unwrap();
        if entry.file_type().is_err() {
            warn!("Skipping entry {entry:?} due to invalid file_type");
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            let dest_dir = to.join(entry.file_name());
            if let Err(e) = fs::create_dir_all(&dest_dir) {
                warn!("Could not create directory {}: {e}", dest_dir.display());
                continue;
            }
            copy_dir_recursive(entry.path().as_path(), &dest_dir);
        } else {
            let from = entry.path();
            let to = to.join(entry.file_name());
            if let Err(e) = fs::copy(&from, &to) {
                warn!("Could not copy {} to {}: {e}", from.display(), to.display());
            } else {
                info!("Copied {} to {}", from.display(), to.display());
            }
        }
    }
}

pub(crate) fn init(ss: SigSet) -> Context {
    create_private_conf_dirs();
    create_shared_conf_dirs();

    /* Copy over initial configuration */
    copy_dir_recursive(Path::new(SHARED_CONF_PATH), Path::new(PRIVATE_CONF_PATH));

    /* Setup inotify watches */
    let mut ctx = Context::default();
    ctx.ss = ss;

    ctx
}

fn handle_file_created(path: &Path) {
    if let Err(e) = fs::copy(
        Path::new(SHARED_CONF_PATH).join(path),
        Path::new(PRIVATE_CONF_PATH).join(path),
    ) {
        error!(
            "Could not copy file {} to {PRIVATE_CONF_PATH}: {e}",
            path.display()
        );
        return;
    }
    nginx::check_reload(Event::MovedTo(path));
}

fn handle_file_deleted(path: &Path) {
    if let Err(e) = fs::remove_file(Path::new(PRIVATE_CONF_PATH).join(path)) {
        error!(
            "Could not remove file {PRIVATE_CONF_PATH}/{}: {e}",
            path.display()
        );
        return;
    }
    nginx::check_reload(Event::Delete(path));
}

pub fn process_events(ctx: &mut Context) {
    let fd = nix::poll::PollFd::new(ctx.inotify.as_fd(), PollFlags::POLLIN);
    let res = ppoll(&mut [fd], None, Some(ctx.ss));
    match res {
        Ok(0) => return,
        Err(e) => {
            warn!("Could not ppoll inotify events: {e}");
            return;
        }
        _ => {}
    }

    let events = ctx.inotify.read_events();
    if let Err(e) = events {
        error!("Could not read inotify events: {e}");
        return;
    }

    for event in events.unwrap() {
        let path = ctx
            .wds
            .get(&event.wd)
            .expect("Events should always be associated with a known wd")
            .join(
                event
                    .name
                    .as_ref()
                    .expect("Event should always be associated with a file"),
            );

        if event.mask.contains(AddWatchFlags::IN_CLOSE_WRITE) {
            if !event.mask.contains(AddWatchFlags::IN_ISDIR) {
                assert!(event.name.is_some());
                println!("{event:?}");
                handle_file_created(&path);
            }
        } else if event.mask.contains(AddWatchFlags::IN_DELETE) {
            if !event.mask.contains(AddWatchFlags::IN_ISDIR) {
                assert!(event.name.is_some());
                println!("{event:?}");
                handle_file_deleted(&path);
            }
        } else {
            panic!("Received unexpected EventMask {:?}", event.mask);
        }
    }
}
