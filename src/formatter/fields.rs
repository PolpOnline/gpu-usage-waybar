use std::{fmt::Debug, str::FromStr};
use strum::{Display, EnumString};

use crate::formatter::units::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Field {
    Simple(SimpleField),
    Mem(MemField, MemUnit),
    Temperature(TemperatureUnit),
    Power(PowerUnit),
    Unknown,
}

impl FromStr for Field {
    type Err = UnitParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let field = if let Ok(f) = SimpleField::from_str(s) {
            Field::Simple(f)
        } else {
            let (field_name, unit_name) = s.split_once(':').ok_or(UnitParseError::NoColon)?;

            if let Ok(f) = MemField::from_str(field_name) {
                Field::Mem(
                    f,
                    MemUnit::from_str(unit_name)
                        .map_err(|_| UnitParseError::Memory(unit_name.to_owned()))?,
                )
            } else if field_name == "temperature" {
                Field::Temperature(
                    TemperatureUnit::from_str(unit_name)
                        .map_err(|_| UnitParseError::Temperature(unit_name.to_owned()))?,
                )
            } else if field_name == "power" {
                Field::Power(
                    PowerUnit::from_str(unit_name)
                        .map_err(|_| UnitParseError::Power(unit_name.to_owned()))?,
                )
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
