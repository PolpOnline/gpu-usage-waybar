use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use serde::Serialize;
use sonic_rs::{JsonValueMutTrait, JsonValueTrait, Value};
use strum::Display;

use crate::config::structs::ConfigFile;

#[derive(Default, Serialize)]
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

impl GpuStatusData {
    pub(crate) fn compute_mem_usage(&self) -> Option<u8> {
        if let (Some(mem_used), Some(mem_total)) = (self.mem_used, self.mem_total) {
            Some((mem_used / mem_total * 100f64).round() as u8)
        } else {
            None
        }
    }

    pub fn get_text(&self, config: &ConfigFile) -> String {
        if !self.powered_on {
            return "Off".to_string();
        }

        let format = &config.text.format;

        self.format_with_fields(format)
    }

    pub fn get_tooltip(&self, config: &ConfigFile) -> String {
        if !self.powered_on {
            return "GPU powered off".to_string();
        }

        let format = &config.tooltip.format;
        self.format_with_fields(format)
    }

    fn format_with_fields(&self, s: &str) -> String {
        let mut value = sonic_rs::to_value(self).unwrap();
        let obj = value.as_object_mut().unwrap();

        if let Some(mem_util) = self.compute_mem_usage() {
            obj.insert("mem_utilization", mem_util);
        }

        let mut result = s.to_string();
        for (key, val) in obj {
            let placeholder = format!("{{{}}}", key);
            let val_str = if val.is_null() {
                "N/A"
            } else {
                &val.to_string()
            };
            result = result.replace(&placeholder, val_str);
        }

        result
    }
}

pub trait GpuStatus {
    fn compute(&self) -> Result<GpuStatusData>;
}

#[derive(Default, Display, Copy, Clone, Serialize)]
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
