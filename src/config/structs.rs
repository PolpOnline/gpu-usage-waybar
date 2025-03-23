use color_eyre::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::Args;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ConfigFile {
    pub general: GeneralConfig,
    pub text_config: TextConfig,
    pub tooltip_config: TooltipConfig,
}

impl ConfigFile {
    pub fn merge_args_into_config(&mut self, args: &Args) -> Result<()> {
        if args.text_no_memory {
            self.text_config = TextConfig { show_memory: false };
        }

        if let Some(interval) = args.interval {
            self.general = GeneralConfig { interval };
        }

        Ok(())
    }
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct TextConfig {
    #[default(true)]
    pub show_memory: bool,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    #[default(1000)]
    pub interval: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TooltipConfig {
    pub gpu_utilization: TooltipTile,
    pub mem_used: TooltipTile,
    pub mem_utilization: TooltipTile,
    pub decoder_utilization: TooltipTile,
    pub encoder_utilization: TooltipTile,
    pub temperature: TooltipTile,
    pub power: TooltipTile,
    pub performance_state: TooltipTile,
    pub performance_level: TooltipTile,
    pub fan_speed: TooltipTile,
    pub tx: TooltipTile,
    pub rx: TooltipTile,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct TooltipTile {
    #[default(true)]
    pub enabled: bool,
    pub icon: String,
    pub text: String,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            gpu_utilization: ("", "GPU").into(),
            mem_used: ("", "MEM USED").into(),
            mem_utilization: ("", "MEM R/W").into(),
            decoder_utilization: ("", "DEC").into(),
            encoder_utilization: ("", "ENC").into(),
            temperature: ("", "TEMP").into(),
            power: ("", "POWER").into(),
            performance_state: ("", "PSTATE").into(),
            performance_level: ("", "PLEVEL").into(),
            fan_speed: ("", "FAN SPEED").into(),
            tx: ("", "TX").into(),
            rx: ("", "RX").into(),
        }
    }
}

impl From<(&str, &str)> for TooltipTile {
    fn from((icon, text): (&str, &str)) -> Self {
        TooltipTile {
            icon: icon.to_string(),
            text: text.to_string(),
            ..Default::default()
        }
    }
}
