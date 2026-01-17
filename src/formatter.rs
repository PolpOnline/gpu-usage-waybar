use regex::Regex;
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Field {
    GpuUtilization,
    MemUsed,
    MemTotal,
    MemRw,
    MemUtilization,
    DecoderUtilization,
    EncoderUtilization,
    Temperature,
    Power,
    PState,
    PLevel,
    FanSpeed,
    Tx,
    Rx,
}

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Static(String),
    Variable(Option<Field>),
}

pub struct State {
    pub chunks: Vec<Chunk>,
    pub buffer: String,
}

impl State {
    pub fn from_format(format: &str) -> State {
        Self {
            chunks: parse(format),
            buffer: String::new(),
        }
    }
}

fn parse(format: &str) -> Vec<Chunk> {
    let re = Regex::new(r"\{([^}]+)}").unwrap();
    let mut chunks = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(format) {
        let m = cap.get(0).unwrap();
        let field_str = &cap[1];

        // static
        chunks.push(Chunk::Static(format[last_end..m.start()].to_string()));

        // variable
        let field = Field::from_str(field_str).ok();
        if field.is_none() {
            eprintln!("Warning: unknown field: {field_str}");
        }

        chunks.push(Chunk::Variable(field));
        last_end = m.end();
    }

    // push the rest static
    chunks.push(Chunk::Static(format[last_end..].to_string()));

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let format = r"PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx} MiB/s
RX: {rx} MiB/s";

        let chunks = parse(format);

        assert_eq!(
            chunks,
            vec![
                Chunk::Static("PSTATE: ".to_string()),
                Chunk::Variable(Some(Field::PState)),
                Chunk::Static("\nPLEVEL: ".to_string()),
                Chunk::Variable(Some(Field::PLevel)),
                Chunk::Static("\nFAN SPEED: ".to_string()),
                Chunk::Variable(Some(Field::FanSpeed)),
                Chunk::Static("%\nTX: ".to_string()),
                Chunk::Variable(Some(Field::Tx)),
                Chunk::Static(" MiB/s\nRX: ".to_string()),
                Chunk::Variable(Some(Field::Rx)),
                Chunk::Static(" MiB/s".to_string()),
            ]
        );
    }
}
