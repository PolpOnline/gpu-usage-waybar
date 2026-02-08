pub mod fields;

use crate::{formatter::State, gpu_status::fields::*, nvidia::PState};
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::Result;
use std::fmt::Write;
use uom::si::{f32::Information, f32::Power};

use crate::formatter::{self, units::*};

pub type Temperature = uom::si::f32::ThermodynamicTemperature;

pub trait GpuStatus {
    fn get_u8_field(&self, _field: U8Field) -> Result<u8, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    fn get_mem_field(&self, _field: MemField) -> Result<Information, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    fn get_power(&self) -> Result<Power, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    fn get_pstate(&self) -> Result<PState, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    fn get_plevel(&self) -> Result<PerformanceLevel, GetFieldError> {
        Err(GetFieldError::BrandUnsupported)
    }
    /// Whether GPU is powered on at the PCI level.
    fn is_powered_on(&self) -> bool {
        // default true so GpuHandle::get_text and GpuHandle::get_tooltip does not skip
        true
    }
    /// Whether any process is using GPU.
    fn has_running_processes(&self) -> bool {
        // default true so GpuHandle::get_text and GpuHandle::get_tooltip does not skip
        true
    }
}

pub struct GpuHandle {
    pub data: Box<dyn GpuStatus>,
}

impl GpuHandle {
    pub fn new(data: Box<dyn GpuStatus>) -> Self {
        Self { data }
    }
    fn compute_mem_usage(&self) -> Option<u8> {
        let (Ok(mem_used), Ok(mem_total)) = (
            self.data.get_mem_field(MemField::MemUsed),
            self.data.get_mem_field(MemField::MemTotal),
        ) else {
            return None;
        };

        let ratio: f32 = (mem_used / mem_total).into();
        Some((ratio * 100.0).round() as u8)
    }

    pub fn get_text<'a>(&self, state: &'a mut State) -> &'a str {
        if !self.data.is_powered_on() {
            return "Off";
        }

        if !self.data.has_running_processes() {
            return "Idle";
        }

        state.assemble(self);
        &state.buffer
    }

    pub fn get_tooltip<'a>(&self, state: &'a mut State) -> &'a str {
        if !self.data.is_powered_on() {
            return "GPU powered off";
        }

        if !self.data.has_running_processes() {
            return "GPU idle";
        }

        state.assemble(self);
        &state.buffer
    }

    /// Write `field` value to `buffer`.
    ///
    /// - Writes "N/A" if `field` is [Field::Unknown].
    /// - Returns [GetFieldError] if `field` is invalid.
    pub fn write_field(&self, field: Field, buffer: &mut String) -> Result<(), GetFieldError> {
        let scan_end_index = buffer.len();

        macro_rules! write_unit {
            ($val:expr, $unit:expr, $precision:expr) => {{
                let v = $unit.compute($val);

                match $precision {
                    Some(precision) => write!(buffer, "{:.*}", precision, v).unwrap(),
                    None => write!(buffer, "{v}").unwrap(),
                }
            }};
        }

        match field {
            Field::U8(field) => write!(buffer, "{}", self.data.get_u8_field(field)?).unwrap(),
            Field::PState => write!(buffer, "{}", self.data.get_pstate()?).unwrap(),
            Field::PLevel => write!(buffer, "{}", self.data.get_plevel()?).unwrap(),
            Field::MemUtilization => write!(
                buffer,
                "{}",
                self.compute_mem_usage().ok_or(GetFieldError::Unavailable)?
            )
            .unwrap(),
            Field::Mem {
                field,
                unit,
                precision,
            } => write_unit!(self.data.get_mem_field(field)?, unit, precision),
            Field::Temperature { unit, precision } => {
                write_unit!(self.data.get_temperature()?, unit, precision)
            }
            Field::Power { unit, precision } => {
                write_unit!(self.data.get_power()?, unit, precision)
            }
            Field::Unknown => buffer.push_str("N/A"),
        };

        formatter::trim_trailing_zeros(buffer, scan_end_index);

        Ok(())
    }
    /// Returns `true` if the field is [Field::Unknown] or the corresponding result is
    /// [GetFieldError].
    pub fn is_field_unavailable(&self, field: Field) -> bool {
        match field {
            Field::Unknown => true,
            Field::U8(field) => self.data.get_u8_field(field).is_err(),

            Field::Mem {
                field,
                unit: _,
                precision: _,
            } => self.data.get_mem_field(field).is_err(),
            Field::Temperature {
                unit: _,
                precision: _,
            } => self.data.get_temperature().is_err(),
            Field::Power {
                unit: _,
                precision: _,
            } => self.data.get_power().is_err(),
            Field::PState => self.data.get_pstate().is_err(),
            Field::PLevel => self.data.get_plevel().is_err(),
            Field::MemUtilization => self.compute_mem_usage().is_none(),
        }
    }
}

#[derive(Debug)]
pub enum GetFieldError {
    Unavailable,
    BrandUnsupported,
}

#[cfg(test)]
mod tests {
    use crate::gpu_status::fields::Field;
    use uom::si::thermodynamic_temperature::degree_celsius;

    use super::*;

    #[test]
    fn test_write_field_precision() {
        struct Data;
        impl GpuStatus for Data {
            fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
                Ok(Temperature::new::<degree_celsius>(35.12345))
            }
        }
        let data = Data;
        let status = GpuHandle::new(Box::new(data));
        let mut buf = String::new();

        status
            .write_field(
                Field::Temperature {
                    unit: TemperatureUnit::Celsius,
                    precision: Some(2),
                },
                &mut buf,
            )
            .unwrap();

        assert_eq!(buf, "35.12");
    }

    #[test]
    fn test_write_field_precision_zero() {
        struct Data;
        impl GpuStatus for Data {
            fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
                Ok(Temperature::new::<degree_celsius>(35.12345))
            }
        }
        let data = Data;
        let status = GpuHandle::new(Box::new(data));
        let mut buf = String::new();

        status
            .write_field(
                Field::Temperature {
                    unit: TemperatureUnit::Celsius,
                    precision: Some(0),
                },
                &mut buf,
            )
            .unwrap();

        assert_eq!(buf, "35");
    }
}
