use color_eyre::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::Args;

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
