use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};
use strum::{Display, EnumString};

use crate::formatter::units::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Field {
    Simple(SimpleField),
    Mem {
        field: MemField,
        unit: MemUnit,
        precision: usize,
    },
    Temperature {
        unit: TemperatureUnit,
        precision: usize,
    },
    Power {
        unit: PowerUnit,
        precision: usize,
    },
    Unknown,
}

impl FromStr for Field {
    type Err = UnitParseError;

    /// Parses a string into a [Field].
    ///
    /// The string can be in the form `p_state` for a [SimpleField], which does not
    /// require a unit, or `temperature:c` when a unit must be specified. The colon
    /// separates the field name and the unit name.
    ///
    /// If the field name (before the colon) is unrecognized, [Field::Unknown] is returned.
    /// If the input is not a [SimpleField] and lacks the required `:unit.precision` syntax,
    /// an error is returned.
    ///
    /// # Errors
    ///
    /// If `field` is not a [SimpleField] and no colon is found in the string,
    /// returns [UnitParseError::NoColon].
    ///
    /// If parsing the unit fails, returns [UnitParseError::Memory],
    /// [UnitParseError::Power], or [UnitParseError::Temperature].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let field = if let Ok(f) = SimpleField::from_str(s) {
            Field::Simple(f)
        } else {
            let (field_name, after) = s.split_once(':').ok_or(UnitParseError::NoColon)?;
            let (unit_name, precision) = after.split_once('.').ok_or(UnitParseError::NoDot)?;
            let precision = usize::from_str(precision)
                .map_err(|_| UnitParseError::Precision(precision.to_owned()))?;

            if let Ok(field) = MemField::from_str(field_name) {
                let unit = MemUnit::from_str(unit_name)
                    .map_err(|_| UnitParseError::Memory(unit_name.to_owned()))?;

                Field::Mem {
                    field,
                    unit,
                    precision,
                }
            } else if field_name == "temperature" {
                let unit = TemperatureUnit::from_str(unit_name)
                    .map_err(|_| UnitParseError::Temperature(unit_name.to_owned()))?;

                Field::Temperature { unit, precision }
            } else if field_name == "power" {
                let unit = PowerUnit::from_str(unit_name)
                    .map_err(|_| UnitParseError::Power(unit_name.to_owned()))?;

                Field::Power { unit, precision }
            } else {
                Field::Unknown
            }
        };

        Ok(field)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SimpleField {
    GpuUtilization,
    MemRw,
    MemUtilization,
    DecoderUtilization,
    EncoderUtilization,
    PState,
    PLevel,
    FanSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MemField {
    MemUsed,
    MemTotal,
    Tx,
    Rx,
}

#[derive(Debug)]
pub enum UnitParseError {
    NoColon,
    NoDot,
    Precision(String),
    Memory(String),
    Temperature(String),
    Power(String),
}

impl Display for UnitParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitParseError::NoColon => write!(
                f,
                "There's no colon found separating field name and unit name."
            ),
            UnitParseError::NoDot => write!(f, "There's no dot found specifying precision."),
            UnitParseError::Precision(s) => write!(f, "Unable to parse precision: `{s}`"),
            UnitParseError::Memory(unit) => write!(f, "Invalid memory unit: `{unit}`"),
            UnitParseError::Temperature(unit) => write!(f, "Invalid temperature unit: `{unit}`"),
            UnitParseError::Power(unit) => write!(f, "Invalid power unit: `{unit}`"),
        }
    }
}

impl Error for UnitParseError {}
