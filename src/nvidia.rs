use std::fs;

use byte_unit::{Byte, Unit};
use color_eyre::eyre::Result;
use nvml_wrapper::{
    Device, Nvml,
    enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor},
};
use procfs::process::{FDTarget, all_processes};

use crate::gpu_status::{GpuStatus, GpuStatusData, PState};

pub struct NvidiaGpuStatus<'a> {
    device: Device<'a>,
    bus_id: String,
}

impl NvidiaGpuStatus<'_> {
    pub fn new(instance: &'static Nvml) -> Result<Self> {
        let device = instance.device_by_index(0)?;

        // Query PCI info just once
        // NVML returns a PCI domain up to 0xffffffff; need to truncate
        // to match sysfs
        let bus_id = device.pci_info()?.bus_id.chars().skip(4).collect();

        Ok(Self { device, bus_id })
    }
}

enum GpuPowerState {
    Off,
    OnNoProcess,
    PoweredOnInUse,
}

fn is_powered_on(bus_id: &str) -> Result<bool> {
    let path = format!("/sys/bus/pci/devices/{bus_id}/power/runtime_status");
    let status = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            // Sometimes the runtime status file doesn't exist or doesn't contain the
            // expected value
            return Ok(true);
        }
    };
    let status = status.trim().to_string();
    let powered_on = status == "active";
    Ok(powered_on)
}

/// Returns `true` if there is any process currently using GPU 0.
///
/// This function checks whether `/dev/nvidia0` is opened by any process
/// other than the current one without waking up the GPU by scanning
/// `/proc/*/fd`.
///
/// # Note
///
/// Do not use
/// [nvml_wrapper::device::Device::running_compute_processes_count] or
/// [nvml_wrapper::device::Device::running_graphics_processes_count]
/// as they wake up the GPU.
///
/// # References
///
/// https://wiki.archlinux.org/title/PRIME#NVIDIA
fn has_running_processes() -> bool {
    let procs = all_processes().expect("Can't read /proc");

    for proc in procs.flatten() {
        if proc.pid == std::process::id() as i32 {
            continue;
        }

        let Ok(fds) = proc.fd() else {
            continue;
        };

        for fd in fds.flatten() {
            if let FDTarget::Path(ref path) = fd.target
                && path == "/dev/nvidia0"
            {
                return true;
            }
        }
    }

    false
}

impl NvidiaGpuStatus<'_> {
    fn detect_gpu_presence(&self) -> Result<GpuPowerState> {
        if !is_powered_on(&self.bus_id)? {
            return Ok(GpuPowerState::Off);
        }

        if !has_running_processes() {
            return Ok(GpuPowerState::OnNoProcess);
        }

        Ok(GpuPowerState::PoweredOnInUse)
    }

    fn collect_active_gpu_stats(&self) -> GpuStatusData {
        let device = &self.device;
        let utilization_rates = device.utilization_rates().ok();
        let memory_info_in_bytes = device.memory_info().ok();

        GpuStatusData {
            powered_on: true,
            has_running_processes: true,
            gpu_utilization: utilization_rates.as_ref().map(|u| u.gpu as u8),
            mem_used: memory_info_in_bytes.as_ref().map(|m| m.used.into()),
            mem_total: memory_info_in_bytes.as_ref().map(|m| m.total.into()),
            mem_rw: utilization_rates.map(|u| u.memory as u8),
            decoder_utilization: device
                .decoder_utilization()
                .ok()
                .map(|u| u.utilization as u8),
            encoder_utilization: device
                .encoder_utilization()
                .ok()
                .map(|u| u.utilization as u8),
            temperature: device
                .temperature(TemperatureSensor::Gpu)
                .ok()
                .map(|t| t as u8),
            power: device.power_usage().ok().map(|p| p as f64 / 1000f64), /* convert to W
                                                                           * from mW */
            p_state: device.performance_state().ok().map(|p| p.into()),
            fan_speed: device.fan_speed(0u32).ok().map(|f| f as u8),
            tx: device
                .pcie_throughput(PcieUtilCounter::Send)
                .ok()
                .map(|t| Byte::from_u64_with_unit(t as u64, Unit::KB).unwrap()),
            rx: device
                .pcie_throughput(PcieUtilCounter::Receive)
                .ok()
                .map(|t| Byte::from_u64_with_unit(t as u64, Unit::KB).unwrap()),
            ..Default::default()
        }
    }
}

impl GpuStatus for NvidiaGpuStatus<'_> {
    fn compute(&self) -> Result<GpuStatusData> {
        // GPU status computation is split into two stages to avoid inadvertently
        // waking up the NVIDIA GPU during idle periods:
        //
        // 1. Presence check (doesn't wake GPU):
        //    - Uses sysfs to check PCI-level power status (`is_powered_on`).
        //    - Scans /proc via procfs to see if any process is currently using the GPU
        //      device node
        //    This stage does not invoke NVML and therefore does not wake the GPU.
        //
        // 2. NVML collection (wake GPU):
        //    - Only executed if the GPU is powered on and has running processes.
        //    - Collects utilization rates, memory info, temperature, PCIe throughput,
        //      encoder/decoder usage, fan speed, power draw, etc.
        //    This stage gives full metrics but is gated to minimize unnecessary GPU
        // wake-ups.
        //
        // By structuring the polling this way, we maintain power-awareness while
        // still collecting full GPU metrics when the device is actively in use.
        let gpu_status = match self.detect_gpu_presence()? {
            GpuPowerState::Off => GpuStatusData {
                powered_on: false,
                has_running_processes: false,
                ..Default::default()
            },
            GpuPowerState::OnNoProcess => GpuStatusData {
                powered_on: true,
                has_running_processes: false,
                ..Default::default()
            },
            GpuPowerState::PoweredOnInUse => self.collect_active_gpu_stats(),
        };

        Ok(gpu_status)
    }

    fn compute_force(&self) -> Result<GpuStatusData> {
        Ok(self.collect_active_gpu_stats())
    }
}

impl From<PerformanceState> for PState {
    fn from(value: PerformanceState) -> Self {
        match value {
            PerformanceState::Zero => PState::P0,
            PerformanceState::One => PState::P1,
            PerformanceState::Two => PState::P2,
            PerformanceState::Three => PState::P3,
            PerformanceState::Four => PState::P4,
            PerformanceState::Five => PState::P5,
            PerformanceState::Six => PState::P6,
            PerformanceState::Seven => PState::P7,
            PerformanceState::Eight => PState::P8,
            PerformanceState::Nine => PState::P9,
            PerformanceState::Ten => PState::P10,
            PerformanceState::Eleven => PState::P11,
            PerformanceState::Twelve => PState::P12,
            PerformanceState::Thirteen => PState::P13,
            PerformanceState::Fourteen => PState::P14,
            PerformanceState::Fifteen => PState::P15,
            PerformanceState::Unknown => PState::Unknown,
        }
    }
}
