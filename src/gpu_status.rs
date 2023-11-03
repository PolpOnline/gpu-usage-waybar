use color_eyre::eyre::Result;
use strum::Display;

#[derive(Default)]
pub struct GpuStatusData {
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
    /// Performance state.
    pub(crate) p_state: Option<PState>,
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
            $target.push_str(&format!(concat!($fmt, "\n"), value));
        }
    };
}

/// Formats the value if it is `Some`, appends it to the `fmt` string,
/// and pushes it to the `target` string.
/// Does not append a newline.
macro_rules! conditional_format_no_newline {
    ($target:ident, $fmt:expr, $($src:expr),*) => {
        $(
            if let Some(value) = $src {
                $target.push_str(&format!($fmt, value));
            }
        )*
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

    pub fn get_text(&self) -> String {
        let mut text = String::new();

        conditional_format_no_newline!(text, "{}%", self.gpu_util);
        text.push('|');
        conditional_format_no_newline!(text, "{}%", self.compute_mem_usage());

        text
    }

    pub fn get_tooltip(&self) -> String {
        let mut tooltip = String::new();

        conditional_format!(tooltip, "GPU: {}%", self.gpu_util);
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
        conditional_format!(tooltip, "MEM R/W: {}%", self.mem_util);
        conditional_format!(tooltip, "DEC: {}%", self.dec_util);
        conditional_format!(tooltip, "ENC: {}%", self.enc_util);
        conditional_format!(tooltip, "TEMP: {}°C", self.temp);
        conditional_format!(tooltip, "POWER: {}W", self.power);
        conditional_format!(tooltip, "PSTATE: {}", self.p_state);
        conditional_format!(tooltip, "FAN SPEED: {}%", self.fan_speed);
        conditional_format!(tooltip, "TX: {} MiB/s", self.tx);
        conditional_format!(tooltip, "RX: {} MiB/s", self.rx);

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
