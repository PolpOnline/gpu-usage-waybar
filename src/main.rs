pub mod amd;
pub mod config;
pub mod gpu_status;
pub mod nvidia;

use std::{
    io::{Write, stdout},
    sync::OnceLock,
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use nvml_wrapper::Nvml;
use serde::Serialize;

use crate::{
    amd::{AmdGpuStatus, AmdSysFS},
    config::structs::ConfigFile,
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
pub struct Args {
    /// Polling interval in milliseconds
    #[arg(long)]
    interval: Option<u64>,

    /// The format you want to display for `text`.
    /// For example,"{gpu_utilization}%|{mem_utilization}%".
    #[arg(long)]
    text_format: Option<String>,

    /// The format you want to display for `tooltip`.
    /// For example,
    /// "GPU: {gpu_utilization}%\n
    /// MEM USED: {mem_used}/{mem_total} MiB ({mem_utilization}%)\n
    /// MEM R/W: {mem_rw}%\n
    /// DEC: {decoder_utilization}%\n
    /// ENC: {encoder_utilization}%\n
    /// TEMP: {temperature}°C\n
    /// POWER: {power}W\n
    /// PSTATE: {p_state}\n
    /// PLEVEL: {p_level}\n
    /// FAN SPEED: {fan_speed}%\n
    /// TX: {tx} MiB/s\n
    /// RX: {rx} MiB/s"
    #[arg(long)]
    tooltip_format: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut config = config::get_or_init_config()?;

    let args = Args::parse();

    config.merge_args_into_config(&args)?;

    let gpu_status_handler: Box<dyn GpuStatus> = match get_instance() {
        Instance::Nvml(nvml) => Box::new(NvidiaGpuStatus::new(nvml)?),
        Instance::Amd(amd_sys_fs) => Box::new(AmdGpuStatus::new(amd_sys_fs)?),
    };

    let update_interval = Duration::from_millis(config.general.interval);

    let mut stdout_lock = stdout().lock();

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        let output = format_output(gpu_status_data, &config);

        writeln!(&mut stdout_lock, "{}", sonic_rs::to_string(&output)?)?;

        std::thread::sleep(update_interval);
    }
}

fn format_output(gpu_status: GpuStatusData, config: &ConfigFile) -> OutputFormat {
    OutputFormat {
        text: gpu_status.get_text(config),
        tooltip: gpu_status.get_tooltip(config),
    }
}

#[derive(Default, Serialize)]
struct OutputFormat {
    text: String,
    tooltip: String,
}
