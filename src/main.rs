pub mod amd;
pub mod gpu_status;
pub mod nvidia;

use std::{
    io::{stdout, Write},
    sync::OnceLock,
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use nvml_wrapper::Nvml;
use serde::Serialize;

use crate::{
    amd::{AmdGpuStatus, AmdSysFS},
    gpu_status::{GpuStatus, GpuStatusData},
    nvidia::NvidiaGpuStatus,
};

pub enum Instance {
    Nvml(Box<Nvml>),
    Amd(Box<AmdSysFS>),
}

impl Instance {
    /// Get the instance based on the GPU brand.
    pub fn new() -> Result<Self> {
        let modules_file = std::fs::read_to_string("/proc/modules")?;

        if modules_file.contains("nvidia") {
            return Ok(Self::Nvml(Box::new(Nvml::init()?)));
        }
        if modules_file.contains("amdgpu") {
            return Ok(Self::Amd(Box::new(AmdSysFS::init()?)));
        }

        Err(eyre!("No supported GPU found"))
    }
}

pub static INSTANCE: OnceLock<Instance> = OnceLock::new();

fn get_instance() -> &'static Instance {
    INSTANCE.get_or_init(|| Instance::new().unwrap())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Add this flag if you don't want to display memory information in the
    /// text output.
    #[arg(long, default_value_t = false)]
    text_no_memory: bool,

    /// Polling interval in milliseconds
    #[arg(long, default_value_t = 1000)]
    interval: u64,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let gpu_status_handler: Box<dyn GpuStatus> = match get_instance() {
        Instance::Nvml(nvml) => Box::new(NvidiaGpuStatus::new(nvml)?),
        Instance::Amd(amd_sys_fs) => Box::new(AmdGpuStatus::new(amd_sys_fs)?),
    };

    let update_interval = Duration::from_millis(args.interval);

    let mut stdout_lock = stdout().lock();

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        let output = format_output(gpu_status_data, !args.text_no_memory);

        writeln!(&mut stdout_lock, "{}", serde_json::to_string(&output)?)?;

        std::thread::sleep(update_interval);
    }
}

fn format_output(gpu_status: GpuStatusData, display_mem_info: bool) -> OutputFormat {
    OutputFormat {
        text: gpu_status.get_text(display_mem_info),
        tooltip: gpu_status.get_tooltip(),
    }
}

#[derive(Default, Serialize)]
struct OutputFormat {
    text: String,
    tooltip: String,
}
