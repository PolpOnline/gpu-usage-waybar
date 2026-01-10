use crate::{
    Args,
    gpu_status::{self, GpuStatusData},
};
use color_eyre::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

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
            self.tooltip.format = tooltip_format.to_owned();
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
    #[default(
        r"GPU: {gpu_utilization}%
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
RX: {rx} MiB/s"
    )]
    pub format: String,
}

impl TooltipConfig {
    pub fn is_default(&self) -> bool {
        self.format == Self::default().format
    }

    /// Retain lines that have available values.
    ///
    /// # Note
    ///
    /// This function checks **only the first variable** referenced in each line.
    /// A line is removed **only if** the first variable evaluates to `None`.
    pub fn retain_lines_with_values(&mut self, data: &GpuStatusData) {
        let mut result = String::new();
        let re = gpu_status::get_regex();

        for line in self.format.split_inclusive('\n') {
            if let Some(caps) = re.captures(line)
                && data.get_field(&caps[1]).is_none()
            {
                continue;
            }

            result.push_str(line);
        }

        self.format = result;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::structs::TooltipConfig,
        gpu_status::{GpuStatusData, PState},
    };

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
            format: r"PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx} MiB/s
RX: {rx} MiB/s"
                .to_string(),
        };

        config.retain_lines_with_values(&data);

        assert_eq!(
            config.format,
            r"PSTATE: {p_state}
TX: {tx} MiB/s
RX: {rx} MiB/s"
        );
    }
}
