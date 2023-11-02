use std::time::Duration;

use color_eyre::eyre::{eyre, Result};
use lazy_static::lazy_static;
use nvidia_smi_waybar::gpu_status::{GpuStatus, GpuStatusData};
use nvml_wrapper::Nvml;
use serde::Serialize;

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

pub enum Instance {
    Nvml(Box<Nvml>),
    Amd(i32),
}

impl Instance {
    /// Get the instance based on the GPU brand.
    pub fn new() -> Result<Self> {
        let modules_file = std::fs::read_to_string("/proc/modules")?;

        if modules_file.contains("nvidia") {
            return Ok(Instance::Nvml(Box::new(Nvml::init()?)));
        }
        if modules_file.contains("amdgpu") {
            return Ok(Instance::Amd(0));
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
        Instance::Nvml(nvml) => Box::new(nvidia_smi_waybar::nvidia::NvidiaGpuStatus::new(nvml)?),
        Instance::Amd(_) => unimplemented!(),
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
