use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use strum::Display;

use crate::config::structs::ConfigFile;

#[derive(Default)]
pub struct GpuStatusData {
    /// Whether GPU is powered on at the PCI level.
    pub(crate) powered_on: bool,
    /// GPU utilization in percent.
    pub(crate) gpu_utilization: Option<u8>,
    /// Memory used in MiB.
    pub(crate) mem_used: Option<f64>,
    /// Total memory in MiB.
    pub(crate) mem_total: Option<f64>,
    /// Memory data bus utilization in percent.
    pub(crate) mem_util: Option<u8>,
    /// Decoder utilization in percent.
    pub(crate) decoder_utilization: Option<u8>,
    /// Encoder utilization in percent.
    pub(crate) encoder_utilization: Option<u8>,
    /// Temperature in degrees Celsius.
    pub(crate) temperature: Option<u8>,
    /// Power usage in Watts.
    pub(crate) power: Option<f64>,
    /// (NVIDIA) Performance state.
    pub(crate) p_state: Option<PState>,
    /// (AMD) Performance Level
    pub(crate) p_level: Option<PerformanceLevel>,
    /// Fan speed in percent.
    pub(crate) fan_speed: Option<u8>,
    /// PCIe TX throughput in MiB/s.
    pub(crate) tx: Option<f64>,
    /// PCIe RX throughput in MiB/s.
    pub(crate) rx: Option<f64>,
}

/// Formats the value if it is `Some`, appends it to the `fmt` string,
/// and pushes it to the `target` string.
fn conditional_append<T: std::fmt::Display>(target: &mut String, fmt: &str, src: Option<T>) {
    if let Some(value) = src {
        target.push_str(&format!("{}{}", fmt, value));
    }
}

impl GpuStatusData {
    pub(crate) fn compute_mem_usage(&self) -> Option<u8> {
        if let (Some(mem_used), Some(mem_total)) = (self.mem_used, self.mem_total) {
            Some((mem_used / mem_total * 100f64).round() as u8)
        } else {
            None
        }
    }

    pub fn get_text(&self, config: &ConfigFile) -> String {
        let mut text = String::new();
        if self.powered_on {
            conditional_append(&mut text, "{}%", self.gpu_utilization);

            if config.text_config.show_memory {
                conditional_append(&mut text, "|{}%", self.compute_mem_usage());
            }
        } else {
            text = "Off".to_string();
        }

        text
    }

    pub fn get_tooltip(&self, _config: &ConfigFile) -> String {
        let mut tooltip = String::new();

        if self.powered_on {
            conditional_append(&mut tooltip, "GPU: {}%\n", self.gpu_utilization);
            if let (Some(mem_used), Some(mem_total), Some(mem_usage)) =
                (self.mem_used, self.mem_total, self.compute_mem_usage())
            {
                tooltip.push_str(&format!(
                    "MEM USED: {}/{} MiB ({}%)\n",
                    mem_used.round(),
                    mem_total,
                    mem_usage
                ));
            }
            conditional_append(&mut tooltip, "MEM R/W: {}%\n", self.mem_util);
            conditional_append(&mut tooltip, "DEC: {}%\n", self.decoder_utilization);
            conditional_append(&mut tooltip, "ENC: {}%\n", self.encoder_utilization);
            conditional_append(&mut tooltip, "TEMP: {}°C\n", self.temperature);
            conditional_append(&mut tooltip, "POWER: {}W\n", self.power);
            conditional_append(&mut tooltip, "PSTATE: {}\n", self.p_state);
            conditional_append(&mut tooltip, "PLEVEL: {}\n", self.p_level);
            conditional_append(&mut tooltip, "FAN SPEED: {}%\n", self.fan_speed);
            conditional_append(&mut tooltip, "TX: {} MiB/s\n", self.tx);
            conditional_append(&mut tooltip, "RX: {} MiB/s\n", self.rx);
        } else {
            tooltip = "GPU powered off".to_string();
        }

        tooltip.trim().to_string()
    }
}

pub trait GpuStatus {
    fn compute(&self) -> Result<GpuStatusData>;
}

#[derive(Default, Display, Copy, Clone)]
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
