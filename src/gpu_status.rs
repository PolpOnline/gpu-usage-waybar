use std::fmt::Display;

use color_eyre::eyre::Result;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor};
use nvml_wrapper::Device;

#[derive(Default)]
pub struct GpuStatus {
    pub(crate) gpu_util: u32,
    pub(crate) mem_used: f64,
    pub(crate) mem_total: f64,
    pub(crate) mem_util: u32,
    pub(crate) dec_util: u32,
    pub(crate) enc_util: u32,
    pub(crate) temp: u32,
    pub(crate) power: f64,
    pub(crate) pstate: PState,
    pub(crate) fan_speed: u32,
    pub(crate) tx: f64,
    pub(crate) rx: f64,
}

impl GpuStatus {
    pub fn new(device: Device) -> Result<Self> {
        let mut gpu_status = GpuStatus::default();

        let utilization_rates = device.utilization_rates()?;
        let memory_info_in_bytes = device.memory_info()?;

        gpu_status.gpu_util = utilization_rates.gpu;
        gpu_status.mem_used = memory_info_in_bytes.used as f64 / 1024f64 / 1024f64;
        gpu_status.mem_total = memory_info_in_bytes.total as f64 / 1024f64 / 1024f64;

        gpu_status.mem_util = utilization_rates.memory;
        gpu_status.dec_util = device.decoder_utilization()?.utilization;
        gpu_status.enc_util = device.encoder_utilization()?.utilization;
        gpu_status.temp = device.temperature(TemperatureSensor::Gpu)?;
        gpu_status.power = device.power_usage()? as f64 / 1000f64; // convert to W from mW
        gpu_status.pstate = device.performance_state()?.into();
        gpu_status.fan_speed = device.fan_speed(0u32)?;
        gpu_status.tx = device.pcie_throughput(PcieUtilCounter::Send)? as f64 / 1000f64;
        gpu_status.rx = device.pcie_throughput(PcieUtilCounter::Receive)? as f64 / 1000f64;

        Ok(gpu_status)
    }

    pub(crate) fn compute_mem_usage(&self) -> u8 {
        let mem_used_percent = (self.mem_used / self.mem_total) * 100f64;
        mem_used_percent.round() as u8
    }

    pub fn get_text(&self) -> String {
        format!("{}%|{}%", self.gpu_util.clone(), self.compute_mem_usage())
    }

    pub fn get_tooltip(&self) -> String {
        format!(
            "GPU: {}%\n\
            MEM USED: {}/{} MiB ({}%)\n\
            MEM R/W: {}%\n\
            DEC: {}%\n\
            ENC: {}%\n\
            TEMP: {}°C\n\
            POWER: {}W\n\
            PSTATE: {}\n\
            FAN SPEED: {}%\n\
            TX: {} MiB/s\n\
            RX: {} MiB/s",
            self.gpu_util,
            self.mem_used.round(),
            self.mem_total,
            self.compute_mem_usage(),
            self.mem_util,
            self.dec_util,
            self.enc_util,
            self.temp,
            self.power,
            self.pstate,
            self.fan_speed,
            self.tx,
            self.rx
        )
    }
}

#[derive(Default)]
pub(crate) enum PState {
    #[default]
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
}

impl Display for PState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pstate = match self {
            PState::P0 => "P0",
            PState::P1 => "P1",
            PState::P2 => "P2",
            PState::P3 => "P3",
            PState::P4 => "P4",
            PState::P5 => "P5",
            PState::P6 => "P6",
            PState::P7 => "P7",
            PState::P8 => "P8",
            PState::P9 => "P9",
            PState::P10 => "P10",
            PState::P11 => "P11",
            PState::P12 => "P12",
            PState::P13 => "P13",
            PState::P14 => "P14",
            PState::P15 => "P15",
        };

        write!(f, "{}", pstate)
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
            _ => PState::default(),
        }
    }
}
