use std::borrow::Cow;
use std::str::FromStr;

use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use strum::Display;

use crate::formatter::{Chunk, Field, State};

#[derive(Default)]
pub struct GpuStatusData {
    /// Whether any process is using GPU.
    pub(crate) has_running_processes: bool,
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

    pub fn get_text<'a>(&self, state: &'a mut State) -> &'a str {
        if !self.powered_on {
            return "Off";
        }

        if !self.has_running_processes {
            return "Idle";
        }

        self.assemble(state);
        &state.buffer
    }

    pub fn get_tooltip<'a>(&self, state: &'a mut State) -> &'a str {
        if !self.powered_on {
            return "GPU powered off";
        }

        if !self.has_running_processes {
            return "GPU idle";
        }

        self.assemble(state);
        &state.buffer
    }

    pub fn get_field_to_string(&self, field: Field) -> Option<String> {
        // Local macro to reduce boilerplate
        macro_rules! s {
            ($val:expr) => {
                $val.map(|v| v.to_string())
            };
        }

        match field {
            Field::GpuUtilization => s!(self.gpu_utilization),
            Field::MemUsed => s!(self.mem_used.map(|v| v.round() as u64)),
            Field::MemTotal => s!(self.mem_total.map(|v| v.round() as u64)),
            Field::MemRw => s!(self.mem_rw),
            Field::MemUtilization => s!(self.compute_mem_usage()),
            Field::DecoderUtilization => s!(self.decoder_utilization),
            Field::EncoderUtilization => s!(self.encoder_utilization),
            Field::Temperature => s!(self.temperature),
            Field::Power => s!(self.power),
            Field::PState => s!(self.p_state),
            Field::PLevel => s!(self.p_level),
            Field::FanSpeed => s!(self.fan_speed),
            Field::Tx => s!(self.tx),
            Field::Rx => s!(self.rx),
        }
    }

    pub fn is_field_unavailable(&self, name: &str) -> bool {
        Field::from_str(name)
            .ok()
            .and_then(|f| self.get_field_to_string(f))
            .is_none()
    }

    fn assemble(&self, state: &mut State) {
        state.buffer.clear();

        for chunk in &state.chunks {
            match chunk {
                Chunk::Static(s) => state.buffer.push_str(s),
                Chunk::Variable(field) => {
                    let s = field
                        .and_then(|f| self.get_field_to_string(f))
                        .map(Cow::Owned)
                        .unwrap_or(Cow::Borrowed("N/A"));

                    state.buffer.push_str(&s);
                }
            }
        }
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
