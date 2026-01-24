use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use std::fmt::{Display, Write};
use strum::Display;
use uom::si::{f32::Information, f32::Power};

use crate::formatter::{fields::*, units::*, *};

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

    /// Write `field` value to `buffer`.
    ///
    /// - Writes "N/A" if `field` is [Field::Unknown].
    /// - Returns [WriteFieldError::FieldIsNone] if `field` is `None`.
    pub fn write_field(&self, field: Field, buffer: &mut String) -> Result<(), WriteFieldError> {
        macro_rules! u {
            ($val:expr, $unit:expr) => {
                if let Some(v) = $val {
                    write!(buffer, "{}", $unit.compute(v)).unwrap();
                } else {
                    return Err(WriteFieldError::FieldIsNone);
                }
            };
        }

        match field {
            Field::Simple(field) => self.write_simple_field(field, buffer)?,
            Field::Mem(field, unit) => u!(self.get_mem_field(field), unit),
            Field::Temperature(unit) => u!(self.temperature, unit),
            Field::Power(unit) => u!(self.power, unit),
            Field::Unknown => buffer.push_str("N/A"),
        };

        Ok(())
    }

    /// Returns `true` if the field is [Field::Unknown] or the corresponding value is `None`.
    pub fn is_field_unavailable(&self, field: Field) -> bool {
        match field {
            Field::Unknown => true,
            Field::Simple(f) => self.get_simple_field_display(f).is_none(),
            Field::Mem(f, _) => self.get_mem_field(f).is_none(),
            Field::Temperature(_) => self.temperature.is_none(),
            Field::Power(_) => self.power.is_none(),
        }
    }

    fn get_simple_field_display(&self, field: SimpleField) -> Option<SimpleDisplay> {
        macro_rules! d {
            ($val:expr) => {
                $val.map(SimpleDisplay::U8)
            };
        }

        match field {
            SimpleField::GpuUtilization => d!(self.gpu_utilization),
            SimpleField::MemRw => d!(self.mem_rw),
            SimpleField::MemUtilization => d!(self.compute_mem_usage()),
            SimpleField::DecoderUtilization => d!(self.decoder_utilization),
            SimpleField::EncoderUtilization => d!(self.encoder_utilization),
            SimpleField::PState => self.p_state.map(SimpleDisplay::PState),
            SimpleField::PLevel => self.p_level.map(SimpleDisplay::PLevel),
            SimpleField::FanSpeed => d!(self.fan_speed),
        }
    }

    fn write_simple_field(
        &self,
        field: SimpleField,
        buffer: &mut String,
    ) -> Result<(), WriteFieldError> {
        if let Some(field_display) = self.get_simple_field_display(field) {
            write!(buffer, "{field_display}").unwrap();
        } else {
            return Err(WriteFieldError::FieldIsNone);
        }

        Ok(())
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

#[derive(Debug)]
pub enum WriteFieldError {
    FieldIsNone,
}

enum SimpleDisplay {
    U8(u8),
    PState(PState),
    PLevel(PerformanceLevel),
}

impl Display for SimpleDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleDisplay::U8(v) => write!(f, "{v}"),
            SimpleDisplay::PState(v) => write!(f, "{v}"),
            SimpleDisplay::PLevel(v) => write!(f, "{v}"),
        }
    }
}
