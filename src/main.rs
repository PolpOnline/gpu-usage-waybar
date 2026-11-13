pub mod amd;
pub mod config;
pub mod formatter;
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
    formatter::State,
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
        let modules = procfs::modules()?;

        if modules.contains_key("nvidia") {
            return Ok(Self::Nvml(Box::new(Nvml::init()?)));
        }
        if modules.contains_key("amdgpu") {
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
    /// MEM USED: {mem_used:MiB}/{mem_total:MiB} MiB ({mem_utilization}%)"
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

    // If the the user didn't set a custom tooltip format,
    // automatically hide any unavailable fields.
    if !config.tooltip.is_format_set() {
        // Fetch the data once to determine which fields are available
        let gpu_status_data = gpu_status_handler.compute_force()?;
        config.tooltip.retain_lines_with_values(&gpu_status_data);
    }

    let mut text_state = State::try_from_format(&config.text.format)?;
    let mut tooltip_state = State::try_from_format(config.tooltip.format())?;

    let update_interval = Duration::from_millis(config.general.interval);

    let mut stdout_lock = stdout().lock();

    // Reused across updates: `write_output` clears it but keeps the allocated
    // capacity, so the serialized JSON does not allocate on every iteration.
    let mut output_buffer = Vec::new();

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        let output = format_output(&gpu_status_data, &mut text_state, &mut tooltip_state);

        write_output(&mut output_buffer, &output)?;
        stdout_lock.write_all(&output_buffer)?;

        std::thread::sleep(update_interval);
    }
}

fn format_output<'t, 'u>(
    gpu_status: &GpuStatusData,
    text_state: &'t mut State,
    tooltip_state: &'u mut State,
) -> OutputFormat<'t, 'u> {
    OutputFormat {
        text: gpu_status.get_text(text_state),
        tooltip: gpu_status.get_tooltip(tooltip_state),
    }
}

#[derive(Default, Serialize)]
struct OutputFormat<'t, 'u> {
    text: &'t str,
    tooltip: &'u str,
}

/// Serialize `output` as a single JSON line into `buffer`.
///
/// `buffer` is cleared first and keeps its allocated capacity between calls.
fn write_output(buffer: &mut Vec<u8>, output: &OutputFormat) -> Result<()> {
    buffer.clear();
    sonic_rs::to_writer(&mut *buffer, output)?;
    buffer.push(b'\n');

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn output_is_a_single_json_line() {
        let output = OutputFormat {
            text: "42%",
            tooltip: "line 1\n\"quoted\"",
        };
        let mut buffer = Vec::new();

        write_output(&mut buffer, &output).unwrap();

        // Literal newlines inside the tooltip must be escaped, otherwise
        // Waybar would receive more than one line per update.
        assert_eq!(buffer.iter().filter(|&&b| b == b'\n').count(), 1);
        assert!(buffer.ends_with(b"\n"));

        let parsed: HashMap<String, String> =
            sonic_rs::from_slice(&buffer[..buffer.len() - 1]).unwrap();
        assert_eq!(parsed["text"], "42%");
        assert_eq!(parsed["tooltip"], "line 1\n\"quoted\"");
    }
}
