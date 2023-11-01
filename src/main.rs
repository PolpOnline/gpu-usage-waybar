use clap::{Parser, ValueEnum};
use std::sync::Mutex;
use std::time::Duration;

use color_eyre::eyre::{anyhow, Result};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use once_cell::sync::OnceCell;
use serde::Serialize;

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Gpu type between AMD and Nvidia
    #[arg(short, long)]
    gpu_type: GpuType,
}

#[derive(Clone, ValueEnum)]
enum GpuType {
    Amd,
    Nvidia,
}

fn main() -> Result<()> {
    let args = Args::parse();

    loop {
        let gpu_status = gpu_status.populate_nvidia(&device)?;

        let output: OutputFormat = gpu_status.into();

        println!("{}", serde_json::to_string(&output)?);

        std::thread::sleep(UPDATE_INTERVAL);
    }
}

mod initializer {
    use super::*;
    fn init_nvidia(gpu_status: GpuStatus) -> Result<()> {
        let nvml = Nvml::init()?;
        let device = nvml.device_by_index(0)?;
        let gpu_status = GpuStatus::populate_nvidia(gpu_status, &device)?;

        let output: OutputFormat = gpu_status.into();

        println!("{}", serde_json::to_string(&output)?);

        Ok(())
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
