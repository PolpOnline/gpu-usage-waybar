use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};
use strum::{Display, EnumString};

use crate::formatter::units::*;

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

impl FromStr for Field {
    type Err = UnitParseError;

    /// Parses a string into a [Field].
    ///
    /// The string can be in the form `p_state`, which does not
    /// require a unit, or `temperature:c` when a unit must be specified. The colon
    /// separates the field name and the unit name. Users can specify decimal places to
    /// display in the form as `temperature:f.2`, which means to display two decimal places.
    ///
    /// If the field name is unrecognized, [Field::Unknown] is returned.
    ///
    /// # Errors
    ///
    /// If the field requires a unit and no colon is found in the string,
    /// returns [UnitParseError::NoUnit].
    ///
    /// If parsing the unit fails, returns [UnitParseError::Memory],
    /// [UnitParseError::Power], or [UnitParseError::Temperature].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let format = FormatSegments::parse(s);

        macro_rules! parse_unit_and_precision {
            ($unit_type:ty, $err_variant:path) => {{
                let unit_name = format.unit.ok_or(UnitParseError::NoUnit)?;
                let unit = <$unit_type>::from_str(unit_name)
                    .map_err(|_| $err_variant(unit_name.to_string()))?;
                let precision = format
                    .precision
                    .map(|p| {
                        p.parse::<usize>()
                            .map_err(|_| UnitParseError::Precision(p.to_string()))
                    })
                    .transpose()?;
                (unit, precision)
            }};
        }

        let field = match format.field {
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
            s => {
                if let Ok(f) = MemField::from_str(s) {
                    let (unit, precision) =
                        parse_unit_and_precision!(MemUnit, UnitParseError::Memory);

                    Field::Mem {
                        field: f,
                        unit,
                        precision,
                    }
                } else {
                    U8Field::from_str(s)
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
    NoUnit,
    Precision(String),
    Memory(String),
    Temperature(String),
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

struct FormatSegments<'a> {
    field: &'a str,
    unit: Option<&'a str>,
    precision: Option<&'a str>,
}

impl<'a> FormatSegments<'a> {
    fn parse(s: &'a str) -> FormatSegments<'a> {
        // Split field and optional unit
        let (field, rest) = match s.split_once(':') {
            Some((f, r)) => (f, Some(r)),
            None => (s, None),
        };

        // Split unit and optional precision
        let (unit, precision) = match rest {
            Some(r) => match r.split_once('.') {
                Some((u, p)) => (Some(u), Some(p)),
                None => (Some(r), None),
            },
            None => (None, None),
        };

        Self {
            field,
            unit,
            precision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format_segments() {
        let seg = FormatSegments::parse("temperature");
        assert_eq!(seg.field, "temperature");
        assert_eq!(seg.unit, None);
        assert_eq!(seg.precision, None);

        let seg = FormatSegments::parse("temperature:c");
        assert_eq!(seg.field, "temperature");
        assert_eq!(seg.unit, Some("c"));
        assert_eq!(seg.precision, None);

        let seg = FormatSegments::parse("temperature:c.2");
        assert_eq!(seg.field, "temperature");
        assert_eq!(seg.unit, Some("c"));
        assert_eq!(seg.precision, Some("2"));
    }
}
