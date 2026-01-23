use regex::Regex;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::{fmt::Debug, str::FromStr};
use strum::{Display, EnumString};
use uom::si::f32::Power;
use uom::si::power::{kilowatt, watt};
use uom::si::thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin};
use uom::si::{
    f32::Information,
    information::{gibibyte, kibibyte, mebibyte},
};

use crate::gpu_status::{GpuStatusData, Temperature};

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

#[derive(Debug)]
pub enum UnitParseError {
    NoColon,
    Memory(String),
    Temperature(String),
    Power(String),
}

impl Display for UnitParseError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for UnitParseError {}

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

pub trait Unit: Copy {
    type Value;

    fn compute(self, v: Self::Value) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum MemUnit {
    KiB,
    MiB,
    GiB,
}

impl Unit for MemUnit {
    type Value = Information;

    fn compute(self, v: Self::Value) -> f32 {
        match self {
            MemUnit::KiB => v.get::<kibibyte>(),
            MemUnit::MiB => v.get::<mebibyte>(),
            MemUnit::GiB => v.get::<gibibyte>(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum TemperatureUnit {
    #[strum(serialize = "c")]
    Celsius,
    #[strum(serialize = "f")]
    Fahrenheit,
    #[strum(serialize = "k")]
    Kelvin,
}

impl Unit for TemperatureUnit {
    type Value = Temperature;

    fn compute(self, v: Self::Value) -> f32 {
        match self {
            TemperatureUnit::Celsius => v.get::<degree_celsius>(),
            TemperatureUnit::Fahrenheit => v.get::<degree_fahrenheit>(),
            TemperatureUnit::Kelvin => v.get::<kelvin>(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum PowerUnit {
    Watt,
    KiloWatt,
}

impl Unit for PowerUnit {
    type Value = Power;

    fn compute(self, v: Self::Value) -> f32 {
        match self {
            PowerUnit::Watt => v.get::<watt>(),
            PowerUnit::KiloWatt => v.get::<kilowatt>(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Static(String),
    Variable(Field),
}

pub struct State {
    pub chunks: Vec<Chunk>,
    pub buffer: String,
}

impl State {
    // TODO: doc
    pub fn assemble(&mut self, data: &GpuStatusData) {
        self.buffer.clear();

        for chunk in &self.chunks {
            match chunk {
                Chunk::Static(s) => self.buffer.push_str(s),
                Chunk::Variable(field) => {
                    let s = data
                        .get_field_to_string(*field)
                        .map_or(Cow::Borrowed("N/A"), Cow::Owned);
                    self.buffer.push_str(&s);
                }
            }
        }
    }
}

impl State {
    pub fn try_from_format(format: &str) -> Result<State, UnitParseError> {
        Ok(Self {
            chunks: parse(format)?,
            buffer: String::new(),
        })
    }
}

pub fn get_regex() -> Regex {
    Regex::new(r"\{([^}]+)}").unwrap()
}

fn parse(format: &str) -> Result<Vec<Chunk>, UnitParseError> {
    let re = get_regex();
    let mut chunks = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(format) {
        let m = cap.get(0).unwrap();
        let s = &cap[1];

        // static
        chunks.push(Chunk::Static(format[last_end..m.start()].to_string()));

        // variable
        let field = Field::from_str(s)?;

        if matches!(field, Field::Unknown) {
            eprintln!("Warning: unknown field: {s}");
        }

        chunks.push(Chunk::Variable(field));
        last_end = m.end();
    }

    // push the rest static
    chunks.push(Chunk::Static(format[last_end..].to_string()));

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let format = r"PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx:MiB} MiB/s
RX: {rx:MiB} MiB/s";

        let chunks = parse(format).unwrap();

        assert_eq!(
            chunks,
            vec![
                Chunk::Static("PSTATE: ".to_string()),
                Chunk::Variable(Field::Simple(SimpleField::PState)),
                Chunk::Static("\nPLEVEL: ".to_string()),
                Chunk::Variable(Field::Simple(SimpleField::PLevel)),
                Chunk::Static("\nFAN SPEED: ".to_string()),
                Chunk::Variable(Field::Simple(SimpleField::FanSpeed)),
                Chunk::Static("%\nTX: ".to_string()),
                Chunk::Variable(Field::Mem(MemField::Tx, MemUnit::MiB)),
                Chunk::Static(" MiB/s\nRX: ".to_string()),
                Chunk::Variable(Field::Mem(MemField::Rx, MemUnit::MiB)),
                Chunk::Static(" MiB/s".to_string()),
            ]
        );
    }
}
