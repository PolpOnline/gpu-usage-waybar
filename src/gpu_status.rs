use color_eyre::eyre::Result;
use nvml_wrapper::Nvml;
use once_cell::sync::OnceCell;

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
    pub(crate) p_state: PState,
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
            self.p_state,
            self.fan_speed,
            self.tx,
            self.rx
        )
    }
}

pub trait GpuStatus {
    fn compute(self) -> Result<GpuStatusData>;
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
