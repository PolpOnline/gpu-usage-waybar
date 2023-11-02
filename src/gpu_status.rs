use color_eyre::eyre::Result;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor};
use nvml_wrapper::{Device, Nvml};
use once_cell::sync::OnceCell;
use std::rc::Rc;
use std::sync::Mutex;
use strum::Display;

trait Instance: Sync + Send {}
impl Instance for Nvml {}
pub static INSTANCE: OnceCell<Box<Mutex<dyn Instance>>> = OnceCell::new();

#[derive(Default)]
pub struct GpuStatusData {
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

impl GpuStatusData {
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

trait GpuStatus {
    fn compute(self) -> Result<GpuStatusData>;
}

struct NvidiaGpuStatus<'a> {
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
            pstate: device.performance_state()?.into(),
            fan_speed: device.fan_speed(0u32)?,
            tx: (device.pcie_throughput(PcieUtilCounter::Send)? as f64 / 1000f64), // convert to MiB/s from KiB/s
            rx: (device.pcie_throughput(PcieUtilCounter::Receive)? as f64 / 1000f64),
        };

        Ok(gpu_status)
    }
}

#[derive(Default, Display)]
pub(crate) enum PState {
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
