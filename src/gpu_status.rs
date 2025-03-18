use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use strum::Display;

#[derive(Default)]
pub struct GpuStatusData {
    /// Whether GPU is powered on at the PCI level.
    pub(crate) powered_on: bool,
    /// GPU utilization in percent.
    pub(crate) gpu_util: Option<u8>,
    /// Memory used in MiB.
    pub(crate) mem_used: Option<f64>,
    /// Total memory in MiB.
    pub(crate) mem_total: Option<f64>,
    /// Memory data bus utilization in percent.
    pub(crate) mem_util: Option<u8>,
    /// Decoder utilization in percent.
    pub(crate) dec_util: Option<u8>,
    /// Encoder utilization in percent.
    pub(crate) enc_util: Option<u8>,
    /// Temperature in degrees Celsius.
    pub(crate) temp: Option<u8>,
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
macro_rules! conditional_format {
    ($target:ident, $fmt:expr, $src:expr) => {
        if let Some(value) = $src {
            $target.push_str(&format!($fmt, value));
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

    pub fn get_text(&self, display_mem_info: bool) -> String {
        let mut text = String::new();
        if self.powered_on {
            conditional_format!(text, "{}%", self.gpu_util);

            if display_mem_info {
                conditional_format!(text, "|{}%", self.compute_mem_usage());
            }
        } else {
            text = "Off".to_string();
        }

        text
    }

    pub fn get_tooltip(&self) -> String {
        let mut tooltip = String::new();

        if self.powered_on {
            conditional_format!(tooltip, "GPU: {}%\n", self.gpu_util);
            if let (Some(mem_used), Some(mem_total), Some(mem_usage)) =
                (self.mem_used, self.mem_total, self.compute_mem_usage())
            {
                tooltip.push_str(&format!(
                    concat!("MEM USED: {}/{} MiB ({}%)", "\n"),
                    mem_used.round(),
                    mem_total,
                    mem_usage
                ));
            }
            conditional_format!(tooltip, "MEM R/W: {}%\n", self.mem_util);
            conditional_format!(tooltip, "DEC: {}%\n", self.dec_util);
            conditional_format!(tooltip, "ENC: {}%\n", self.enc_util);
            conditional_format!(tooltip, "TEMP: {}°C\n", self.temp);
            conditional_format!(tooltip, "POWER: {}W\n", self.power);
            conditional_format!(tooltip, "PSTATE: {}\n", self.p_state);
            conditional_format!(tooltip, "PLEVEL: {}\n", self.p_level);
            conditional_format!(tooltip, "FAN SPEED: {}%\n", self.fan_speed);
            conditional_format!(tooltip, "TX: {} MiB/s\n", self.tx);
            conditional_format!(tooltip, "RX: {} MiB/s\n", self.rx);
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
