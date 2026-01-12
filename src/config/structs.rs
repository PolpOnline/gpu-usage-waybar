use color_eyre::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{
    Args,
    gpu_status::{self, GpuStatusData},
};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ConfigFile {
    pub general: GeneralConfig,
    pub text: TextConfig,
    pub tooltip: TooltipConfig,
}

impl ConfigFile {
    pub fn merge_args_into_config(&mut self, args: &Args) -> Result<()> {
        if let Some(interval) = args.interval {
            self.general.interval = interval;
        }
        if let Some(ref text_format) = args.text_format {
            self.text.format = text_format.to_owned();
        }
        if let Some(ref tooltip_format) = args.tooltip_format {
            self.tooltip.format = Some(tooltip_format.to_owned());
        }

        Ok(())
    }
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TextConfig {
    #[default("{gpu_utilization}%|{mem_utilization}%")]
    pub format: String,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct GeneralConfig {
    #[default(1000)]
    pub interval: u64,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TooltipConfig {
    format: Option<String>,
}

impl TooltipConfig {
    pub const DEFAULT_FORMAT: &str = r"GPU: {gpu_utilization}%
MEM USED: {mem_used}/{mem_total} MiB ({mem_utilization}%)
MEM R/W: {mem_rw}%
DEC: {decoder_utilization}%
ENC: {encoder_utilization}%
TEMP: {temperature}°C
POWER: {power}W
PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx} MiB/s
RX: {rx} MiB/s";

    pub fn format(&self) -> &str {
        self.format.as_deref().unwrap_or(Self::DEFAULT_FORMAT)
    }

    pub fn is_format_set(&self) -> bool {
        self.format.is_some()
    }

    /// Retain lines that have available values in the format string.
    ///
    /// # Note
    ///
    /// This function modifies the `format` field in place.
    /// If a line contains **any** placeholder without a corresponding value
    /// in `data`, that entire line is removed from the format.
    pub fn retain_lines_with_values(&mut self, data: &GpuStatusData) {
        let mut result = String::new();
        let re = gpu_status::get_regex();

        for line in self.format().split_inclusive('\n') {
            // Check if ANY placeholder in the line has no value
            let has_unavailable = re
                .captures_iter(line)
                .any(|caps| data.get_field(&caps[1]).is_none());

            if has_unavailable {
                continue;
            }

            result.push_str(line);
        }

        self.format = Some(result);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::structs::TooltipConfig,
        gpu_status::{GpuStatusData, PState},
    };

    /// Test that lines with unavailable fields are dropped.
    #[test]
    fn test_retain_some_fields() {
        let data = GpuStatusData {
            p_state: Some(PState::P0),
            p_level: None,
            fan_speed: None,
            tx: Some(5.2),
            rx: Some(6.7),
            ..Default::default()
        };

        let mut config = TooltipConfig {
            format: Some(
                r"PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx} MiB/s
RX: {rx} MiB/s"
                    .to_string(),
            ),
        };

        config.retain_lines_with_values(&data);

        assert_eq!(
            config.format.unwrap(),
            r"PSTATE: {p_state}
TX: {tx} MiB/s
RX: {rx} MiB/s"
        );
    }

    /// Test that lines with multiple placeholders are dropped if any of them
    /// have no value.
    #[test]
    fn test_retain_lines_with_multiple_placeholders() {
        let data = GpuStatusData {
            gpu_utilization: Some(50),
            mem_used: Some(50.0),
            mem_total: None, // This should cause the line to be dropped
            p_state: Some(PState::P0),
            p_level: None, // This should cause the line to be dropped
            ..Default::default()
        };

        let format = r"GPU: {gpu_utilization}% | MEM: {mem_utilization}%
+PSTATE: {p_state} | PLEVEL: {p_level}";

        let mut config = TooltipConfig {
            format: Some(format.to_string()),
        };

        config.retain_lines_with_values(&data);
        // Both lines should be dropped because each has at least one unavailable field
        assert_eq!(config.format, Some("".to_string()));
    }
}
