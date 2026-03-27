use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};
use strum::{Display, EnumString};

use crate::formatter::{FormatSegments, units::*};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Field {
    U8(U8Field),
    Mem {
        field: MemField,
        unit: MemUnit,
        precision: Option<usize>,
    },
    Temperature {
        unit: TemperatureUnit,
        precision: Option<usize>,
    },
    Power {
        unit: PowerUnit,
        precision: Option<usize>,
    },
    /// (NVIDIA) Performance state.
    PState,
    /// (AMD) Performance Level
    PLevel,
    /// Memory utilization in percent computed as [MemField::MemUsed] / [MemField::MemTotal].
    MemUtilization,
    Unknown,
}

impl TryFrom<FormatSegments<'_>> for Field {
    type Error = UnitParseError;

    /// Parses a [FormatSegments] into a [Field].
    ///
    /// If the `segments.field` is unrecognized, [Field::Unknown] is returned.
    fn try_from(segments: FormatSegments<'_>) -> Result<Self, Self::Error> {
        macro_rules! parse_unit_and_precision {
            ($unit_type:ty, $err_variant:path) => {{
                let unit_name = segments.unit.ok_or(UnitParseError::NoUnit)?;
                let unit = <$unit_type>::from_str(unit_name)
                    .map_err(|_| $err_variant(unit_name.to_string()))?;
                let precision = segments
                    .precision
                    .map(|p| {
                        p.parse::<usize>()
                            .map_err(|_| UnitParseError::Precision(p.to_string()))
                    })
                    .transpose()?;
                (unit, precision)
            }};
        }

        let field = match segments.field {
            "p_level" => Field::PLevel,
            "p_state" => Field::PState,
            "mem_utilization" => Field::MemUtilization,
            "power" => {
                let (unit, precision) = parse_unit_and_precision!(PowerUnit, UnitParseError::Power);
                Field::Power { unit, precision }
            }
            "temperature" => {
                let (unit, precision) =
                    parse_unit_and_precision!(TemperatureUnit, UnitParseError::Temperature);
                Field::Temperature { unit, precision }
            }
            field_name => {
                if let Ok(f) = MemField::from_str(field_name) {
                    let (unit, precision) =
                        parse_unit_and_precision!(MemUnit, UnitParseError::Memory);

                    Field::Mem {
                        field: f,
                        unit,
                        precision,
                    }
                } else {
                    U8Field::from_str(field_name)
                        .map(Field::U8)
                        .unwrap_or(Field::Unknown)
                }
            }
        };

        Ok(field)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum U8Field {
    /// GPU utilization in percent.
    GpuUtilization,
    /// Render engine utilization in percent.
    RenderUtilization,
    /// Video engine utilization in percent.
    VideoUtilization,
    /// Memory data bus utilization in percent.
    MemRw,
    /// Decoder utilization in percent.
    DecoderUtilization,
    /// Encoder utilization in percent.
    EncoderUtilization,
    /// Fan speed in percent.
    FanSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MemField {
    /// Memory used.
    MemUsed,
    /// Total memory.
    MemTotal,
    /// PCIe TX throughput per second.
    Tx,
    /// PCIe RX throughput per second.
    Rx,
}

#[derive(Debug)]
pub enum UnitParseError {
    /// Corresponding field requires a unit, but not provided.
    NoUnit,
    /// Error parsing precision numbers.
    Precision(String),
    /// Error parsing memory unit.
    Memory(String),
    /// Error parsing temperature unit.
    Temperature(String),
    /// Error parsing power unit.
    Power(String),
}

impl Display for UnitParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitParseError::NoUnit => write!(f, "No unit provided where required"),
            UnitParseError::Precision(s) => write!(f, "Unable to parse precision: `{s}`"),
            UnitParseError::Memory(unit) => write!(f, "Invalid memory unit: `{unit}`"),
            UnitParseError::Temperature(unit) => write!(f, "Invalid temperature unit: `{unit}`"),
            UnitParseError::Power(unit) => write!(f, "Invalid power unit: `{unit}`"),
        }
    }
}

impl Error for UnitParseError {}
