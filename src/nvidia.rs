use std::{ffi::OsString, fs, path::PathBuf};

use color_eyre::eyre;
use nvml_wrapper::{
    Device, Nvml,
    enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor},
};
use procfs::process::{FDTarget, ProcessesIter};
use strum::Display;
use uom::si::{
    f32::Information,
    f32::Power,
    information::{byte, kilobyte},
    power::milliwatt,
    thermodynamic_temperature::degree_celsius,
};

use crate::gpu_status::{GetFieldError, fields::*};
use crate::gpu_status::{GpuStatus, Temperature};

pub struct NvidiaGpuStatus {
    nvml: Nvml,
    pci_bus_id: String,
    runtime_status_path: PathBuf,
    has_running_procs: bool,
    devnames: Box<[OsString]>,
}

impl NvidiaGpuStatus {
    pub fn new(pci_bus_id: String, devnames: Box<[OsString]>) -> eyre::Result<Self> {
        let nvml = Nvml::init()?;
        let runtime_status_path = format!(
            "/sys/bus/pci/devices/{}/power/runtime_status",
            pci_bus_id.as_str()
        )
        .into();

        Ok(Self {
            nvml,
            pci_bus_id,
            runtime_status_path,
            has_running_procs: true,
            devnames,
        })
    }

    fn device(&self) -> Device<'_> {
        self.nvml
            .device_by_pci_bus_id(self.pci_bus_id.as_str())
            .unwrap()
    }
}

impl GpuStatus for NvidiaGpuStatus {
    fn get_u8_field(&self, field: U8Field) -> Result<u8, GetFieldError> {
        let maybe_val = match field {
            U8Field::GpuUtilization => self
                .device()
                .utilization_rates()
                .as_ref()
                .map(|u| u.gpu as u8)
                .ok(),
            U8Field::MemRw => self
                .device()
                .utilization_rates()
                .as_ref()
                .map(|r| r.memory as u8)
                .ok(),
            U8Field::DecoderUtilization => self
                .device()
                .decoder_utilization()
                .map(|u| u.utilization as u8)
                .ok(),
            U8Field::EncoderUtilization => self
                .device()
                .encoder_utilization()
                .map(|u| u.utilization as u8)
                .ok(),
            U8Field::FanSpeed => self.device().fan_speed(0u32).map(|f| f as u8).ok(),
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        maybe_val.ok_or(GetFieldError::Unavailable)
    }

    fn get_mem_field(&self, field: MemField) -> Result<Information, GetFieldError> {
        let maybe_val = match field {
            MemField::MemUsed => self
                .device()
                .memory_info()
                .as_ref()
                .map(|m| Information::new::<byte>(m.used as f32))
                .ok(),
            MemField::MemTotal => self
                .device()
                .memory_info()
                .as_ref()
                .map(|m| Information::new::<byte>(m.total as f32))
                .ok(),
            MemField::Tx => self
                .device()
                .pcie_throughput(PcieUtilCounter::Send)
                .map(|t| Information::new::<kilobyte>(t as f32))
                .ok(),
            MemField::Rx => self
                .device()
                .pcie_throughput(PcieUtilCounter::Receive)
                .map(|t| Information::new::<kilobyte>(t as f32))
                .ok(),
        };

        maybe_val.ok_or(GetFieldError::Unavailable)
    }

    fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
        self.device()
            .temperature(TemperatureSensor::Gpu)
            .map(|t| Temperature::new::<degree_celsius>(t as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_power(&self) -> Result<Power, GetFieldError> {
        self.device()
            .power_usage()
            .map(|p| Power::new::<milliwatt>(p as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_pstate(&self) -> Result<PState, GetFieldError> {
        self.device()
            .performance_state()
            .map(|p| p.into())
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn is_powered_on(&self) -> bool {
        let status = match fs::read_to_string(&self.runtime_status_path) {
            Ok(s) => s,
            Err(_) => {
                // Sometimes the runtime status file doesn't exist or doesn't contain the
                // expected value
                return true;
            }
        };

        status.trim() == "active"
    }

    fn has_running_processes(&self) -> bool {
        self.has_running_procs
    }

    fn update(&mut self, procs: ProcessesIter) -> eyre::Result<()> {
        self.has_running_procs = has_running_processes(procs, &self.devnames);
        Ok(())
    }
}

#[derive(Default, Display, Copy, Clone)]
pub enum PState {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
    P9,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
    #[default]
    Unknown,
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

/// Returns `true` if any process is currently using a GPU.
///
/// This function checks whether any device in `devnames` is opened by any process.
/// It performs this check without waking up the GPU by scanning file descriptors.
///
/// # Note
///
/// Avoid using [`nvml_wrapper::device::Device::running_compute_processes_count`] or
/// [`nvml_wrapper::device::Device::running_graphics_processes_count`],
/// as these methods will wake up the GPU.
///
/// While [1] suggests checking `/dev/nvidia*`, we haven't find out how device index `*`
/// in multi-GPU systems is determined. Instead, this function checks `devnames`
/// (typically `card*` and `renderD*`).
///
/// We observed that monitoring tools like `nvtop` and `gpu-usage-waybar`, which
/// only use NVML and do not perform GPU computation, only open `nvidia*` and
/// do not open `card*/renderD*`. Conversely, any process performing GPU computation
/// will open `card*/renderD*`.
///
/// [1]: https://wiki.archlinux.org/title/PRIME#NVIDIA
fn has_running_processes(procs: ProcessesIter, devnames: &[OsString]) -> bool {
    for proc in procs.flatten() {
        let Ok(fds) = proc.fd() else {
            continue;
        };

        for fd in fds.flatten() {
            if let FDTarget::Path(ref path) = fd.target
                && (devnames.iter().any(|n| n == path.file_name().unwrap()))
            {
                return true;
            }
        }
    }

    false
}
