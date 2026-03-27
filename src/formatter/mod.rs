pub mod units;

use regex::Regex;
use std::fmt::Debug;

use crate::gpu_status::{
    GpuHandle,
    fields::{Field, UnitParseError},
};

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
    /// Assembles `self.chunks` into `self.buffer` using the provided `data`.
    ///
    /// Writes `"N/A"` if a variable segment in `chunks` is [`Field::Unknown`],
    /// or if the corresponding field in `data` is `None`.
    pub fn assemble(&mut self, handle: &GpuHandle) {
        self.buffer.clear();

        for chunk in &self.chunks {
            match chunk {
                Chunk::Static(s) => self.buffer.push_str(s),
                Chunk::Variable(field) => {
                    // write_field() writes "N/A" if field is Field::Unknown.
                    if handle.write_field(*field, &mut self.buffer).is_err() {
                        self.buffer.push_str("N/A");
                    }
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

#[derive(Debug, Clone, Copy)]
pub struct FormatSegments<'a> {
    pub field: &'a str,
    pub unit: Option<&'a str>,
    pub precision: Option<&'a str>,
}

impl<'a> FormatSegments<'a> {
    /// `caps` must be the captures from [get_regex].
    pub fn from_caps_unchecked(caps: &'a regex::Captures<'_>) -> FormatSegments<'a> {
        FormatSegments {
            field: &caps[1],
            unit: caps.get(2).map(|v| v.as_str()),
            precision: caps.get(3).map(|v| v.as_str()),
        }
    }
}

pub fn get_regex() -> Regex {
    Regex::new(r"\{(\w+)(?::(\w+)(?:\.(\d+))?)?\}").unwrap()
}

pub fn trim_trailing_zeros(buf: &mut String, scan_end_index: usize) {
    let Some(last_dot_pos) = buf.rfind('.') else {
        return;
    };

    // do not trim zeros if the last dot pos is before scan_end_index
    if last_dot_pos <= scan_end_index {
        return;
    }

    let mut end = buf.len();

    while end > last_dot_pos + 1 && buf.as_bytes()[end - 1] == b'0' {
        end -= 1;
    }

    // If only '.' left
    if end == last_dot_pos + 1 {
        end -= 1;
    }

    buf.truncate(end);
}
fn parse(format: &str) -> Result<Vec<Chunk>, UnitParseError> {
    let re = get_regex();
    let mut chunks = Vec::new();
    let mut last_end = 0;

    for caps in re.captures_iter(format) {
        let m = caps.get(0).unwrap();
        let format_segments = FormatSegments::from_caps_unchecked(&caps);

        // static
        if m.start() > last_end {
            chunks.push(Chunk::Static(format[last_end..m.start()].to_string()));
        }

        // variable
        let field = Field::try_from(format_segments)?;

        if matches!(field, Field::Unknown) {
            eprintln!("Warning: unknown field: {}", format_segments.field);
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
    use crate::{
        formatter::units::{MemUnit, TemperatureUnit},
        gpu_status::fields::*,
    };

    #[test]
    fn test_parse() {
        let format = r"PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx:MiB.1} MiB/s
RX: {rx:MiB.2} MiB/s";

        let chunks = parse(format).unwrap();

        assert_eq!(
            chunks,
            vec![
                Chunk::Static("PSTATE: ".to_string()),
                Chunk::Variable(Field::PState),
                Chunk::Static("\nPLEVEL: ".to_string()),
                Chunk::Variable(Field::PLevel),
                Chunk::Static("\nFAN SPEED: ".to_string()),
                Chunk::Variable(Field::U8(U8Field::FanSpeed)),
                Chunk::Static("%\nTX: ".to_string()),
                Chunk::Variable(Field::Mem {
                    field: MemField::Tx,
                    unit: MemUnit::MiB,
                    precision: Some(1),
                }),
                Chunk::Static(" MiB/s\nRX: ".to_string()),
                Chunk::Variable(Field::Mem {
                    field: MemField::Rx,
                    unit: MemUnit::MiB,
                    precision: Some(2),
                }),
                Chunk::Static(" MiB/s".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_unit() {
        assert!(matches!(
            parse("{temperature}"),
            Err(UnitParseError::NoUnit)
        ));

        let field = &parse("{temperature:c}").unwrap()[0];
        assert!(matches!(
            field,
            Chunk::Variable(Field::Temperature {
                unit: TemperatureUnit::Celsius,
                precision: None
            })
        ));

        let field = &parse("{temperature:c.2}").unwrap()[0];
        assert!(matches!(
            field,
            Chunk::Variable(Field::Temperature {
                unit: TemperatureUnit::Celsius,
                precision: Some(2)
            })
        ));
    }

    #[test]
    fn test_trim_trailing_zeros() {
        let mut buf = "1.50000".to_string();
        trim_trailing_zeros(&mut buf, 0);
        assert_eq!(buf, "1.5");
    }

    #[test]
    fn test_trim_trailing_zeros_and_dot() {
        let mut buf = "1.00000".to_string();
        trim_trailing_zeros(&mut buf, 0);
        assert_eq!(buf, "1");
    }

    #[test]
    fn test_trim_trailing_zeros_without_decimal() {
        let mut buf = "10000".to_string();
        trim_trailing_zeros(&mut buf, 0);
        assert_eq!(buf, "10000");
    }

    #[test]
    fn test_trim_trailing_zeros_with_previous_decimals() {
        let mut buf = "100.00 120".to_string();
        trim_trailing_zeros(&mut buf, 7);
        assert_eq!(buf, "100.00 120");
    }

    #[test]
    fn test_trim_trailing_zeros_with_mutiple_dots() {
        let mut buf = "100.00 120.0 500.000".to_string();
        trim_trailing_zeros(&mut buf, 13);
        assert_eq!(buf, "100.00 120.0 500");
    }
}
