use color_eyre::eyre::Result;
use strum::Display;

#[derive(Default)]
pub struct GpuStatusData {
    /// GPU utilization in percent.
    pub(crate) gpu_util: u8,
    /// Memory used in MiB.
    pub(crate) mem_used: f64,
    /// Total memory in MiB.
    pub(crate) mem_total: f64,
    /// Memory utilization in percent.
    pub(crate) mem_util: u8,
    /// Decoder utilization in percent.
    pub(crate) dec_util: u8,
    /// Encoder utilization in percent.
    pub(crate) enc_util: u8,
    /// Temperature in degrees Celsius.
    pub(crate) temp: u8,
    /// Power usage in Watts.
    pub(crate) power: f64,
    /// Performance state.
    pub(crate) p_state: PState,
    /// Fan speed in percent.
    pub(crate) fan_speed: u8,
    /// PCIe TX throughput in MiB/s.
    pub(crate) tx: f64,
    /// PCIe RX throughput in MiB/s.
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
            self.p_state,
            self.fan_speed,
            self.tx,
            self.rx
        )
    }
}

pub trait GpuStatus {
    fn compute(&self) -> Result<GpuStatusData>;
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
