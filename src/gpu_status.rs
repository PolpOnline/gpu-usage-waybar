use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use strum::Display;
use uom::si::{f32::Information, f32::Power};

use crate::formatter::{units::*, *};

pub type Temperature = uom::si::f32::ThermodynamicTemperature;

#[derive(Default)]
pub struct GpuStatusData {
    /// Whether any process is using GPU.
    pub(crate) has_running_processes: bool,
    /// Whether GPU is powered on at the PCI level.
    pub(crate) powered_on: bool,
    /// GPU utilization in percent.
    pub(crate) gpu_utilization: Option<u8>,
    /// Memory used.
    pub(crate) mem_used: Option<Information>,
    /// Total memory.
    pub(crate) mem_total: Option<Information>,
    /// Memory data bus utilization in percent.
    pub(crate) mem_rw: Option<u8>,
    /// Decoder utilization in percent.
    pub(crate) decoder_utilization: Option<u8>,
    /// Encoder utilization in percent.
    pub(crate) encoder_utilization: Option<u8>,
    /// Temperature.
    pub(crate) temperature: Option<Temperature>,
    /// Power usage.
    pub(crate) power: Option<Power>,
    /// (NVIDIA) Performance state.
    pub(crate) p_state: Option<PState>,
    /// (AMD) Performance Level
    pub(crate) p_level: Option<PerformanceLevel>,
    /// Fan speed in percent.
    pub(crate) fan_speed: Option<u8>,
    /// PCIe TX throughput per second.
    pub(crate) tx: Option<Information>,
    /// PCIe RX throughput per second.
    pub(crate) rx: Option<Information>,
}

impl GpuStatusData {
    pub(crate) fn compute_mem_usage(&self) -> Option<u8> {
        if let (Some(mem_used), Some(mem_total)) = (self.mem_used, self.mem_total) {
            let ratio: f32 = (mem_used / mem_total).into();
            Some((ratio * 100.0).round() as u8)
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

        state.assemble(self);
        &state.buffer
    }

    pub fn get_tooltip<'a>(&self, state: &'a mut State) -> &'a str {
        if !self.powered_on {
            return "GPU powered off";
        }

        if !self.has_running_processes {
            return "GPU idle";
        }

        state.assemble(self);
        &state.buffer
    }

    // TODO: doc
    pub fn get_field_to_string(&self, field: Field) -> Option<String> {
        macro_rules! u {
            ($val:expr, $unit:expr) => {
                $val.map(|v| $unit.compute(v).to_string())
            };
        }

        match field {
            Field::Simple(field) => self.get_simple_field_to_string(field),
            Field::Mem(field, unit) => u!(self.get_mem_field(field), unit),
            Field::Temperature(unit) => u!(self.temperature, unit),
            Field::Power(unit) => u!(self.power, unit),
            Field::Unknown => None,
        }
    }

    pub fn is_field_unavailable(&self, field: Field) -> bool {
        self.get_field_to_string(field).is_none()
    }

    fn get_simple_field_to_string(&self, field: SimpleField) -> Option<String> {
        // Local macro to reduce boilerplate
        macro_rules! s {
            ($val:expr) => {
                $val.map(|v| v.to_string())
            };
        }

        match field {
            SimpleField::GpuUtilization => s!(self.gpu_utilization),
            SimpleField::MemRw => s!(self.mem_rw),
            SimpleField::MemUtilization => s!(self.compute_mem_usage()),
            SimpleField::DecoderUtilization => s!(self.decoder_utilization),
            SimpleField::EncoderUtilization => s!(self.encoder_utilization),
            SimpleField::PState => s!(self.p_state),
            SimpleField::PLevel => s!(self.p_level),
            SimpleField::FanSpeed => s!(self.fan_speed),
        }
    }

    fn get_mem_field(&self, field: MemField) -> Option<Information> {
        match field {
            MemField::MemUsed => self.mem_used,
            MemField::MemTotal => self.mem_total,
            MemField::Tx => self.tx,
            MemField::Rx => self.rx,
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
