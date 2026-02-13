use std::fs;

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
    bus_id: String,
    has_running_procs: bool,
}

impl NvidiaGpuStatus {
    pub fn new() -> eyre::Result<Self> {
        let nvml = Nvml::init()?;
        let device = nvml.device_by_index(0)?;
        // Query PCI info just once
        // NVML returns a PCI domain up to 0xffffffff; need to truncate
        // to match sysfs
        let bus_id = device.pci_info()?.bus_id.chars().skip(4).collect();

        Ok(Self {
            nvml,
            bus_id,
            has_running_procs: true,
        })
    }

    fn device(&self) -> Device<'_> {
        self.nvml.device_by_index(0).unwrap()
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
        let path = format!("/sys/bus/pci/devices/{}/power/runtime_status", self.bus_id);
        let status = match fs::read_to_string(path) {
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
        self.has_running_procs = has_running_processes(procs);
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
/// <https://wiki.archlinux.org/title/PRIME#NVIDIA>
fn has_running_processes(procs: ProcessesIter) -> bool {
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
