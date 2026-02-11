pub mod amd;
pub mod config;
pub mod drm;
pub mod formatter;
pub mod gpu_status;
pub mod nvidia;

use std::{
    io::{Write, stdout},
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use udev::Hwdb;

use crate::{
    amd::AmdGpuStatus, drm::DrmDevice, formatter::State, gpu_status::GpuHandle,
    nvidia::NvidiaGpuStatus,
};

fn get_handle(gpu: &DrmDevice, hwdb: &Hwdb) -> Result<GpuHandle> {
    let vendor_name = gpu
        .get_vendor_name(hwdb)?
        .into_string()
        .unwrap()
        .to_lowercase();

    // we could use equal to match vendor, but using contains is always safer
    // vendor_name == "Nvidia Corporation"
    if vendor_name.contains("nvidia") {
        return Ok(GpuHandle::new(Box::new(NvidiaGpuStatus::new()?)));
    }
    // vendor_name == "Advanced Micro Devices, Inc. [AMD/ATI]"
    if vendor_name.contains("advanced micro devices") {
        return Ok(GpuHandle::new(Box::new(AmdGpuStatus::new(
            gpu.device.syspath().to_path_buf(),
        )?)));
    }
    // vendor_name == "Intel Corporation"
    if vendor_name.contains("intel") {
        todo!();
    }

    Err(eyre!("No supported GPU found"))
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The GPU index you want to monitor.
    /// The index is typically the `X` in /dev/dri/cardX.
    #[arg(long, default_value = "0")]
    gpu: usize,

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

    let gpus = drm::scan_drm_devices()?;
    let gpu = gpus
        .get(args.gpu)
        .ok_or(eyre!("Cannot find GPU {}", args.gpu))?;
    let hwdb = Hwdb::new()?;
    print_gpu(args.gpu, gpu, &hwdb)?;
    let gpu_status_handle = get_handle(gpu, &hwdb)?;

    // If the the user didn't set a custom tooltip format,
    // automatically hide any unavailable fields.
    if !config.tooltip.is_format_set() {
        config.tooltip.retain_lines_with_values(&gpu_status_handle);
    }

    let mut text_state = State::try_from_format(&config.text.format)?;
    let mut tooltip_state = State::try_from_format(config.tooltip.format())?;

    let update_interval = Duration::from_millis(config.general.interval);

    let mut stdout_lock = stdout().lock();

    loop {
        let output = format_output(&gpu_status_handle, &mut text_state, &mut tooltip_state);

        writeln!(&mut stdout_lock, "{}", sonic_rs::to_string(&output)?)?;

        std::thread::sleep(update_interval);
    }
}

fn print_gpu(gpu_index: usize, gpu: &DrmDevice, hwdb: &Hwdb) -> Result<()> {
    print!(
        "GPU {}: {}, Nodes: ",
        gpu_index,
        gpu.get_model_name(hwdb)?.to_str().unwrap()
    );

    let mut nodes = gpu.children.iter().map(|dev| dev.sysname());
    let first = nodes.next().unwrap().to_str().unwrap().to_owned();
    let nodes = nodes.fold(first, |a, b| {
        format!("{}, {}", a.as_str(), b.to_str().unwrap())
    });
    println!("{nodes}");

    Ok(())
}

fn format_output<'t, 'u>(
    handle: &GpuHandle,
    text_state: &'t mut State,
    tooltip_state: &'u mut State,
) -> OutputFormat<'t, 'u> {
    OutputFormat {
        text: handle.get_text(text_state),
        tooltip: handle.get_tooltip(tooltip_state),
    }
}

#[derive(Default, Serialize)]
struct OutputFormat<'t, 'u> {
    text: &'t str,
    tooltip: &'u str,
}
