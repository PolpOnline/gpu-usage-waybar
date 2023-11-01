use std::sync::Mutex;

use color_eyre::eyre::{anyhow, Result};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use once_cell::sync::OnceCell;
use serde::Serialize;

use nvidia_smi_waybar::gpu_status::GpuStatus;

static NVML_INSTANCE: OnceCell<Mutex<core::result::Result<Nvml, NvmlError>>> = OnceCell::new();

fn main() -> Result<()> {
    let nvml = NVML_INSTANCE
        .get_or_init(|| {
            let nvml = Nvml::init();
            Mutex::new(nvml)
        })
        .lock()
        .unwrap();

    let nvml = nvml
        .as_ref()
        .map_err(|e| anyhow!("Failed to initialize NVML {}", e))?;

    let device = nvml.device_by_index(0)?;

    loop {
        let gpu_status = GpuStatus::new(&device)?;

        let output: OutputFormat = gpu_status.into();

        println!("{}", serde_json::to_string(&output)?);

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

impl From<GpuStatus> for OutputFormat {
    fn from(gpu_status: GpuStatus) -> Self {
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
