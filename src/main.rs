pub mod amd;
pub mod gpu_status;
pub mod nvidia;

use std::time::Duration;

use crate::amd::{AmdGpuStatus, AmdSysFS};
use crate::gpu_status::{GpuStatus, GpuStatusData};
use crate::nvidia::NvidiaGpuStatus;
use color_eyre::eyre::{eyre, Result};
use lazy_static::lazy_static;
use nvml_wrapper::Nvml;
use serde::Serialize;

/// Polling interval
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

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

lazy_static! {
    pub static ref INSTANCE: Instance = Instance::new().unwrap();
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let gpu_status_handler: Box<dyn GpuStatus> = match &*INSTANCE {
        Instance::Nvml(nvml) => Box::new(NvidiaGpuStatus::new(nvml)?),
        Instance::Amd(amd_sys_fs) => Box::new(AmdGpuStatus::new(amd_sys_fs)?),
    };

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        let output: OutputFormat = gpu_status_data.into();

        println!("{}", serde_json::to_string(&output)?);

        std::thread::sleep(UPDATE_INTERVAL);
    }
}

impl From<GpuStatusData> for OutputFormat {
    fn from(gpu_status: GpuStatusData) -> OutputFormat {
        OutputFormat {
            text: gpu_status.get_text(),
            tooltip: gpu_status.get_tooltip(),
        }
    }
}

#[derive(Default, Serialize)]
struct OutputFormat {
    text: String,
    tooltip: String,
}
