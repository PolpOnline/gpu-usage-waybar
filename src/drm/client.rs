use std::{
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    time::Instant,
};

use procfs::process::{FDInfo, FDTarget, Process, ProcessesIter};

pub struct DrmClient {
    pub render_engine: EngineStats,
    // TODO: other engines
    reader: BufReader<File>,
    id: u32,
    last_seen: u64,
}

const RENDER_ENGINE_KEY: &str = "drm-engine-render";

impl DrmClient {
    fn update_engines(&mut self) -> io::Result<()> {
        let reader = &mut self.reader;
        reader.seek(SeekFrom::Start(0))?;

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(RENDER_ENGINE_KEY) {
                let value = line.split_whitespace().nth(1).unwrap().parse().unwrap();
                let sample = EngineSample::new(value);
                self.render_engine.update_utilization(sample);
                break;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct ClientManager {
    pub clients: Vec<DrmClient>,
    devnames: Box<[OsString]>,
    current_tick: u64,
}

impl ClientManager {
    pub fn new(devnames: Box<[OsString]>) -> Self {
        Self {
            devnames,
            clients: Vec::new(),
            current_tick: 0,
        }
    }

    pub fn update(&mut self, procs: ProcessesIter) {
        self.current_tick += 1;

        for proc in procs.flatten() {
            self.scan_process_fds(proc);
        }

        self.clients.retain(|c| c.last_seen == self.current_tick);

        for client in self.clients.iter_mut() {
            client.update_engines().unwrap();
        }
    }

    fn scan_process_fds(&mut self, proc: Process) {
        let Ok(fds) = proc.fd() else { return };

        for fd in fds.flatten() {
            if !self.should_manage(&fd) {
                continue;
            }
            let Ok(fdinfo_file) = File::open(format!("/proc/{}/fdinfo/{}", proc.pid, fd.fd)) else {
                continue;
            };
            let mut reader = BufReader::new(fdinfo_file);

            if let Some(id) = read_id(&mut reader) {
                self.mark_or_insert_client(id, reader);
            }
        }
    }

    fn mark_or_insert_client(&mut self, id: u32, reader: BufReader<File>) {
        if let Some(client) = self.clients.iter_mut().find(|c| c.id == id) {
            client.last_seen = self.current_tick;
        } else {
            self.clients.push(DrmClient {
                render_engine: EngineStats::default(),
                reader,
                id,
                last_seen: self.current_tick,
            });
        }
    }

    fn should_manage(&self, fd: &FDInfo) -> bool {
        let FDTarget::Path(target) = &fd.target else {
            return false;
        };
        self.devnames
            .iter()
            .any(|n| n == target.file_name().unwrap())
    }
}

#[derive(Default)]
pub struct EngineStats {
    pub utilization: Option<f64>,
    last_sample: Option<EngineSample>,
}

impl EngineStats {
    fn update_utilization(&mut self, sample: EngineSample) {
        if let Some(last_sample) = self.last_sample {
            let delta_used = sample.value - last_sample.value;
            let delta_sample = sample
                .sample_finished_at
                .duration_since(last_sample.sample_finished_at)
                .as_nanos();

            self.utilization = Some(delta_used as f64 / delta_sample as f64);
        }

        self.last_sample = Some(sample);
    }
}

#[derive(Clone, Copy)]
struct EngineSample {
    value: u64,
    sample_finished_at: Instant,
}

impl EngineSample {
    fn new(value: u64) -> Self {
        Self {
            value,
            sample_finished_at: Instant::now(),
        }
    }
}

fn read_id(reader: &mut BufReader<File>) -> Option<u32> {
    reader
        .lines()
        .map_while(Result::ok)
        .find(|l| l.starts_with("drm-client-id"))
        .map(|l| l.split_whitespace().nth(1).unwrap().parse().unwrap())
}
