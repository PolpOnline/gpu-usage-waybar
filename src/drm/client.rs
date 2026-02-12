use std::{
    fs::{self, File, Metadata},
    io::{BufRead, BufReader, Read},
    os::linux::fs::MetadataExt,
};

use procfs::process::{FDTarget, ProcessesIter};

pub struct DrmClient {
    reader: BufReader<File>,
    id: u32,
    last_seen: u64,
}

impl DrmClient {
    fn read_id<R: ?Sized + Read>(reader: &mut BufReader<R>) -> Option<u32> {
        reader
            .lines()
            .map_while(Result::ok)
            .find(|l| l.starts_with("drm-client-id"))
            .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
    }
}

#[derive(Default)]
struct ClientManager {
    clients: Vec<DrmClient>,
    current_tick: u64,
}

impl ClientManager {
    fn update(&mut self, procs: ProcessesIter) {
        self.current_tick += 1;

        for proc in procs.flatten() {
            let Ok(fds) = proc.fd() else { continue };

            for fd in fds.flatten() {
                let FDTarget::Path(ref path) = fd.target else {
                    continue;
                };
                let Ok(meta) = fs::metadata(path) else {
                    continue;
                };
                if !is_drm_fd(&meta) {
                    continue;
                };
                let Ok(fdinfo) = File::open(format!("/proc/{}/fdinfo/{}", proc.pid, fd.fd)) else {
                    continue;
                };

                let mut reader = BufReader::new(fdinfo);
                let Some(id) = DrmClient::read_id(&mut reader) else {
                    continue;
                };

                let current_tick = self.current_tick;
                if let Some(client) = self.clients.iter_mut().find(|client| client.id == id) {
                    client.last_seen = current_tick;
                } else {
                    self.clients.push(DrmClient {
                        reader,
                        id,
                        last_seen: current_tick,
                    });
                }

                self.clients
                    .retain(|client| client.last_seen == current_tick);
            }
        }
    }
}

const DRM_DEVNODE_MAJOR: u32 = 226;

fn is_drm_fd(metadata: &Metadata) -> bool {
    let is_char_dev = metadata.st_mode() & libc::S_IFMT == libc::S_IFCHR;
    let is_drm_dev = libc::major(metadata.st_rdev()) == DRM_DEVNODE_MAJOR;
    is_char_dev && is_drm_dev
}

#[test]
fn find_drm_clients() {
    let mut mgr = ClientManager::default();
    mgr.update(procfs::process::all_processes().unwrap());

    for client in mgr.clients {
        dbg!(client.id);
    }
}
