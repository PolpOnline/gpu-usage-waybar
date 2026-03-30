pub mod amd;
pub mod config;
pub mod formatter;
pub mod gpu_status;
pub mod nvidia;

use std::io::{self, Write};
use std::{io::stdout, sync::OnceLock, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use nvml_wrapper::Nvml;

use crate::gpu_status::GpuStatusData;
use crate::{
    amd::{AmdGpuStatus, AmdSysFS},
    formatter::State,
    gpu_status::GpuStatus,
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

    // Escape special chars in the formats to make the JSON output valid.
    // Also make line breaks literal (\n -> \\n), because we don't want
    // to flush stdout before the whole JSON content is ready in the stdout buffer.
    let escaped_text_format = json_escape_simd::escape(&config.text.format);
    let escaped_tooltip_format = json_escape_simd::escape(config.tooltip.format());

    let text_state = State::try_from_format(escaped_text_format)?;
    let tooltip_state = State::try_from_format(escaped_tooltip_format)?;

    let update_interval = Duration::from_millis(config.general.interval);

    let mut stdout_lock = stdout().lock();

    loop {
        let gpu_status_data = gpu_status_handler.compute()?;

        // Static string chunks in `text_state` and `tooltip_state`
        // were properly escaped with `json_escape_simd` before the loop.
        // **Variable chunks should not yields special characters that
        // should be escaped, either, which is unchecked.**
        write_json_unchecked(
            &mut stdout_lock,
            &gpu_status_data,
            &text_state,
            &tooltip_state,
        )?;

        std::thread::sleep(update_interval);
    }
}

/// Write `data` to waybar-flavored json:
/// ```json
/// {"text": "...", "tooltip": "..."}
/// ```
///
/// This function does not escape characters for you.
/// Callers have to make sure `text_state` and `tooltip_state`
/// does not contains strings such as `\n`, `"`, ... that
/// could break JSON.
fn write_json_unchecked(
    buffer: &mut impl Write,
    data: &GpuStatusData,
    text_state: &State,
    tooltip_state: &State,
) -> io::Result<()> {
    write!(buffer, r#"{{"text":""#)?;
    data.write_text(text_state, buffer)?;
    write!(buffer, r#"","tooltip":""#)?;
    data.write_tooltip(tooltip_state, buffer)?;
    writeln!(buffer, r#""}}"#)?;

    Ok(())
}
