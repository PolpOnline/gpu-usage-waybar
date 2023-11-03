use crate::gpu_status::{GpuStatus, GpuStatusData, PState};
use color_eyre::eyre::Result;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor};
use nvml_wrapper::{Device, Nvml};

pub struct NvidiaGpuStatus<'a> {
    device: Device<'a>,
}

impl NvidiaGpuStatus<'_> {
    pub fn new(instance: &'static Nvml) -> Result<Self> {
        let device = instance.device_by_index(0)?;

        Ok(Self { device })
    }
}

impl GpuStatus for NvidiaGpuStatus<'_> {
    fn compute(&self) -> Result<GpuStatusData> {
        let device = &self.device;

        let utilization_rates = device.utilization_rates()?;
        let memory_info_in_bytes = device.memory_info()?;

        let gpu_status = GpuStatusData {
            gpu_util: Some(utilization_rates.gpu as u8),
            mem_used: Some(memory_info_in_bytes.used as f64 / 1024f64 / 1024f64), // convert to MiB from B
            mem_total: Some(memory_info_in_bytes.total as f64 / 1024f64 / 1024f64),
            mem_util: Some(utilization_rates.memory as u8),
            dec_util: Some(device.decoder_utilization()?.utilization as u8),
            enc_util: Some(device.encoder_utilization()?.utilization as u8),
            temp: Some(device.temperature(TemperatureSensor::Gpu)? as u8),
            power: Some(device.power_usage()? as f64 / 1000f64), // convert to W from mW
            p_state: Some(device.performance_state()?.into()),
            fan_speed: Some(device.fan_speed(0u32)? as u8),
            tx: Some(device.pcie_throughput(PcieUtilCounter::Send)? as f64 / 1000f64), // convert to MiB/s from KiB/s
            rx: Some(device.pcie_throughput(PcieUtilCounter::Receive)? as f64 / 1000f64),
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
