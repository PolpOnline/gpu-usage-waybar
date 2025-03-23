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
    pub(crate) mem_rw: Option<u8>,
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
macro_rules! conditional_append {
    ($target:ident, $fmt:expr, $value:expr) => {
        if let Some(value) = $value {
            $target.push_str(&format!($fmt, value));
        }
    };

    // Same but with two values
    ($target:ident, $fmt:expr, $src1:expr, $src2:expr) => {
        if let (Some(value1), Some(value2)) = ($src1, $src2) {
            $target.push_str(&format!($fmt, value1, value2));
        }
    };

    // Same but with four values
    ($target:ident, $fmt:expr, $src1:expr, $src2:expr, $src3:expr, $src4:expr) => {
        if let (Some(value1), Some(value2), Some(value3), Some(value4)) =
            ($src1, $src2, $src3, $src4)
        {
            $target.push_str(&format!($fmt, value1, value2, value3, value4));
        }
    };
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
            conditional_append!(text, "{}%", self.gpu_utilization);

            if config.text.show_memory {
                conditional_append!(text, "|{}%", self.compute_mem_usage());
            }
        } else {
            text = "Off".to_string();
        }

        text
    }

    pub fn get_tooltip(&self, config_file: &ConfigFile) -> String {
        let config = &config_file.tooltip;

        let mut tooltip = String::new();

        if self.powered_on {
            conditional_append!(
                tooltip,
                "{}: {}%\n",
                config.gpu_utilization.get_text(),
                self.gpu_utilization
            );
            conditional_append!(
                tooltip,
                "{}: {}/{} MiB ({}%)\n",
                config.mem_utilization.get_text(),
                self.mem_used.map(|v| v.round() as u64),
                self.mem_total.map(|v| v.round() as u64),
                self.compute_mem_usage()
            );
            conditional_append!(tooltip, "{}: {} %\n", config.mem_rw.get_text(), self.mem_rw);
            conditional_append!(
                tooltip,
                "{}: {} %\n",
                config.decoder_utilization.get_text(),
                self.decoder_utilization
            );
            conditional_append!(
                tooltip,
                "{}: {} %\n",
                config.encoder_utilization.get_text(),
                self.encoder_utilization
            );
            conditional_append!(
                tooltip,
                "{}: {} °C\n",
                config.temperature.get_text(),
                self.temperature
            );
            conditional_append!(tooltip, "{}: {} W\n", config.power.get_text(), self.power);
            conditional_append!(tooltip, "{}: {}\n", config.p_state.get_text(), self.p_state);
            conditional_append!(tooltip, "{}: {}\n", config.p_level.get_text(), self.p_level);
            conditional_append!(
                tooltip,
                "{}: {} %\n",
                config.fan_speed.get_text(),
                self.fan_speed
            );
            conditional_append!(tooltip, "{}: {} MiB/s\n", config.tx.get_text(), self.tx);
            conditional_append!(tooltip, "{}: {} MiB/s\n", config.rx.get_text(), self.rx);
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
