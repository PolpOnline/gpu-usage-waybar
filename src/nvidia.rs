use crate::gpu_status::{GpuStatus, GpuStatusData, PState};
use color_eyre::eyre::Result;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor};
use nvml_wrapper::{Device, Nvml};
use std::rc::Rc;

pub struct NvidiaGpuStatus<'a> {
    nvml: Rc<Nvml>,
    device: Device<'a>,
}

impl NvidiaGpuStatus<'_> {
    pub fn new() -> Result<Self> {
        let nvml = Rc::new(Nvml::init()?);

        let device = nvml.clone().device_by_index(0)?;

        Ok(Self { nvml, device })
    }
}

impl GpuStatus for NvidiaGpuStatus<'_> {
    fn compute(self) -> Result<GpuStatusData> {
        let device = self.device;

        let utilization_rates = device.utilization_rates()?;
        let memory_info_in_bytes = device.memory_info()?;

        let gpu_status = GpuStatusData {
            gpu_util: utilization_rates.gpu,
            mem_used: (memory_info_in_bytes.used as f64 / 1024f64 / 1024f64),
            mem_total: (memory_info_in_bytes.total as f64 / 1024f64 / 1024f64),
            mem_util: utilization_rates.memory,
            dec_util: device.decoder_utilization()?.utilization,
            enc_util: device.encoder_utilization()?.utilization,
            temp: device.temperature(TemperatureSensor::Gpu)?,
            power: (device.power_usage()? as f64 / 1000f64), // convert to W from mW
            p_state: device.performance_state()?.into(),
            fan_speed: device.fan_speed(0u32)?,
            tx: (device.pcie_throughput(PcieUtilCounter::Send)? as f64 / 1000f64), // convert to MiB/s from KiB/s
            rx: (device.pcie_throughput(PcieUtilCounter::Receive)? as f64 / 1000f64),
        };

        Ok(gpu_status)
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
