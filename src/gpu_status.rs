use std::{borrow::Cow, sync::OnceLock};

use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use byte_unit::{AdjustedByte, Byte, Unit, UnitParseError};
use color_eyre::eyre::Result;
use regex::Regex;
use strum::Display;

use crate::config::structs::ConfigFile;

static RE: OnceLock<Regex> = OnceLock::new();

pub fn get_regex() -> &'static Regex {
    RE.get_or_init(|| Regex::new(r"\{([^}]+)}").unwrap())
}

#[derive(Default)]
pub struct GpuStatusData {
    /// Whether any process is using GPU.
    pub(crate) has_running_processes: bool,
    /// Whether GPU is powered on at the PCI level.
    pub(crate) powered_on: bool,
    /// GPU utilization in percent.
    pub(crate) gpu_utilization: Option<u8>,
    /// Memory used.
    pub(crate) mem_used: Option<Byte>,
    /// Total memory.
    pub(crate) mem_total: Option<Byte>,
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
    /// PCIe TX throughput per second.
    pub(crate) tx: Option<Byte>,
    /// PCIe RX throughput per second.
    pub(crate) rx: Option<Byte>,
}

impl GpuStatusData {
    pub(crate) fn compute_mem_usage(&self) -> Option<u8> {
        if let (Some(mem_used), Some(mem_total)) = (self.mem_used, self.mem_total) {
            let ratio = mem_used.as_u64() as f64 / mem_total.as_u64() as f64;
            Some((ratio * 100f64).round() as u8)
        } else {
            None
        }
    }

    pub fn get_text(&self, config: &ConfigFile) -> String {
        if !self.powered_on {
            return "Off".to_string();
        }

        if !self.has_running_processes {
            return "Idle".to_string();
        }

        let format = &config.text.format;
        self.format_with_fields(format)
    }

    pub fn get_tooltip(&self, config: &ConfigFile) -> String {
        if !self.powered_on {
            return "GPU powered off".to_string();
        }

        if !self.has_running_processes {
            return "GPU idle".to_string();
        }

        let format = &config.tooltip.format();
        self.format_with_fields(format)
    }

    pub fn get_field(&self, name: &str) -> Option<String> {
        // Local macro to reduce boilerplate
        macro_rules! s {
            ($val:expr) => {
                $val.map(|v| v.to_string())
            };
        }

        match name {
            "gpu_utilization" => s!(self.gpu_utilization),
            "mem_rw" => s!(self.mem_rw),
            "mem_utilization" => s!(self.compute_mem_usage()),
            "decoder_utilization" => s!(self.decoder_utilization),
            "encoder_utilization" => s!(self.encoder_utilization),
            "temperature" => s!(self.temperature),
            "power" => s!(self.power),
            "p_state" => s!(self.p_state),
            "p_level" => s!(self.p_level),
            "fan_speed" => s!(self.fan_speed),
            _ => {
                // TODO: Handle digits
                let maybe_byte_field = self.format_byte_field(name);

                if maybe_byte_field.is_none() {
                    eprintln!("Warning: unknown field: {}", name);
                }

                maybe_byte_field.map(|byte| byte.get_value().to_string())
            }
        }
    }

    /// Returns an [AdjustedByte] in the specified unit.
    /// The unit is determined by the template suffix.
    /// For example, it adjusts to `MiB` for the template `{mem_used_MiB}`.
    ///
    /// Returns `None` if the name prefix does not match any field, or if
    /// the unit cannot be parsed.
    fn format_byte_field(&self, name: &str) -> Option<AdjustedByte> {
        let fields = [
            ("mem_used_", self.mem_used),
            ("mem_total_", self.mem_total),
            ("tx_", self.tx),
            ("rx_", self.rx),
        ];

        for (prefix, value) in fields {
            if let Some(v) = value
                && name.starts_with(prefix)
            {
                return Self::get_adjusted_unit(name, prefix, v).ok();
            }
        }

        None
    }

    fn get_adjusted_unit(
        name: &str,
        prefix: &str,
        byte: Byte,
    ) -> Result<AdjustedByte, UnitParseError> {
        let unit_str = name.strip_prefix(prefix).unwrap();
        let unit = Unit::parse_str(unit_str, true, true)?;

        Ok(byte.get_adjusted_unit(unit))
    }

    fn format_with_fields(&self, s: &str) -> String {
        // Regex to match patterns like {variable_name}
        let re = get_regex();

        re.replace_all(s, |caps: &regex::Captures| {
            let key = &caps[1];
            self.get_field(key)
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed("N/A"))
        })
        .into_owned()
    }
}

pub trait GpuStatus {
    fn compute(&self) -> Result<GpuStatusData>;

    /// Compute [GpuStatusData] regardless of idle or power state.
    fn compute_force(&self) -> Result<GpuStatusData> {
        self.compute()
    }
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
