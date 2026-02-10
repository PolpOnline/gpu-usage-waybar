use std::fmt::Debug;
use strum::{Display, EnumString};
use uom::si::{
    f32::{Information, Power},
    information::*,
    power::{kilowatt, watt},
    thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin},
};

use crate::gpu_status::Temperature;

pub trait Unit: Copy {
    type Value;

    fn compute(self, v: Self::Value) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum MemUnit {
    KiB,
    MiB,
    GiB,
    KB,
    MB,
    GB,
    Kib,
    Mib,
    Gib,
    Kb,
    Mb,
    Gb,
}

impl Unit for MemUnit {
    type Value = Information;

    fn compute(self, v: Self::Value) -> f32 {
        match self {
            MemUnit::KiB => v.get::<kibibyte>(),
            MemUnit::MiB => v.get::<mebibyte>(),
            MemUnit::GiB => v.get::<gibibyte>(),
            MemUnit::KB => v.get::<kilobyte>(),
            MemUnit::MB => v.get::<megabyte>(),
            MemUnit::GB => v.get::<gigabyte>(),
            MemUnit::Kib => v.get::<kibibit>(),
            MemUnit::Mib => v.get::<mebibit>(),
            MemUnit::Gib => v.get::<gibibit>(),
            MemUnit::Kb => v.get::<kilobit>(),
            MemUnit::Mb => v.get::<megabit>(),
            MemUnit::Gb => v.get::<gigabit>(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(ascii_case_insensitive)]
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
    #[strum(serialize = "w")]
    Watt,
    #[strum(serialize = "kw")]
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
