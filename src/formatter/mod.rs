pub mod fields;
pub mod units;

use regex::Regex;
use std::{fmt::Debug, str::FromStr};

use crate::{
    formatter::{fields::*, units::*},
    gpu_status::{GpuStatusData, WriteFieldError},
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
    pub fn assemble(&mut self, data: &GpuStatusData) {
        self.buffer.clear();

        for chunk in &self.chunks {
            match chunk {
                Chunk::Static(s) => self.buffer.push_str(s),
                Chunk::Variable(field) => {
                    if matches!(
                        // write_field() writes "N/A" if field is Field::Unknown.
                        data.write_field(*field, &mut self.buffer),
                        Err(WriteFieldError::FieldIsNone)
                    ) {
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

pub fn get_regex() -> Regex {
    Regex::new(r"\{([^}]+)}").unwrap()
}

pub fn trim_trailing_zeros(buf: &mut String) {
    let Some(dot_pos) = buf.find('.') else {
        return;
    };

    let mut end = buf.len();

    while end > dot_pos + 1 && buf.as_bytes()[end - 1] == b'0' {
        end -= 1;
    }

    // If only '.' left
    if end == dot_pos + 1 {
        end -= 1;
    }

    buf.truncate(end);
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
TX: {tx:MiB.1} MiB/s
RX: {rx:MiB.2} MiB/s";

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
                Chunk::Variable(Field::Mem {
                    field: MemField::Tx,
                    unit: MemUnit::MiB,
                    precision: 1,
                }),
                Chunk::Static(" MiB/s\nRX: ".to_string()),
                Chunk::Variable(Field::Mem {
                    field: MemField::Rx,
                    unit: MemUnit::MiB,
                    precision: 2,
                }),
                Chunk::Static(" MiB/s".to_string()),
            ]
        );
    }

    #[test]
    fn test_trim_trailing_zeros() {
        let mut buf = "1.50000".to_string();
        trim_trailing_zeros(&mut buf);
        assert_eq!(buf, "1.5");
    }

    #[test]
    fn test_trim_trailing_zeros_and_dot() {
        let mut buf = "1.00000".to_string();
        trim_trailing_zeros(&mut buf);
        assert_eq!(buf, "1");
    }
}
