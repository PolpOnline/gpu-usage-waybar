use std::env::Args;
use std::time::Duration;

use color_eyre::eyre::Result;
use nvidia_smi_waybar::clap::GpuType;
use nvidia_smi_waybar::gpu_status::{GpuStatus, GpuStatusData};
use nvidia_smi_waybar::nvidia::NvidiaGpuStatus;
use serde::Serialize;

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let gpu_status_handler: dyn GpuStatus = match args.gpu_type {
        GpuType::Amd => todo!(),
        GpuType::Nvidia => NvidiaGpuStatus::new()?,
    };

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        let output: OutputFormat = gpu_status_data.into();

        println!("{}", serde_json::to_string(&output)?);

        std::thread::sleep(UPDATE_INTERVAL);
    }
}

impl From<GpuStatusData> for OutputFormat {
    fn from(gpu_status: GpuStatusData) -> Self {
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
