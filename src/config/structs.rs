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
    pub fn merge_args_into_config(&mut self, args: &Args) -> color_eyre::Result<()> {
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TooltipTile {
    pub enabled: bool,
    pub icon: String,
    pub text: String,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            gpu_utilization: TooltipTile::new("".to_string(), "GPU".to_string()),
            mem_used: TooltipTile::new("".to_string(), "MEM USED".to_string()),
            mem_utilization: TooltipTile::new("".to_string(), "MEM R/W".to_string()),
            decoder_utilization: TooltipTile::new("".to_string(), "DEC".to_string()),
            encoder_utilization: TooltipTile::new("".to_string(), "ENC".to_string()),
            temperature: TooltipTile::new("".to_string(), "TEMP".to_string()),
            power: TooltipTile::new("".to_string(), "POWER".to_string()),
            performance_state: TooltipTile::new("".to_string(), "PSTATE".to_string()),
            performance_level: TooltipTile::new("".to_string(), "PLEVEL".to_string()),
            fan_speed: TooltipTile::new("".to_string(), "FAN SPEED".to_string()),
            tx: TooltipTile::new("".to_string(), "TX".to_string()),
            rx: TooltipTile::new("".to_string(), "RX".to_string()),
        }
    }
}

impl TooltipTile {
    pub fn new(icon: String, text: String) -> Self {
        TooltipTile {
            enabled: true,
            icon,
            text,
        }
    }
}
