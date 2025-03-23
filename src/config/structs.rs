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
        if args.text_no_memory {
            self.text = TextConfig { show_memory: false };
        }

        if let Some(interval) = args.interval {
            self.general = GeneralConfig { interval };
        }

        Ok(())
    }
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TextConfig {
    #[default(true)]
    pub show_memory: bool,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct GeneralConfig {
    #[default(1000)]
    pub interval: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TooltipConfig {
    pub gpu_utilization: TooltipTile,
    pub mem_utilization: TooltipTile,
    pub mem_rw: TooltipTile,
    pub decoder_utilization: TooltipTile,
    pub encoder_utilization: TooltipTile,
    pub temperature: TooltipTile,
    pub power: TooltipTile,
    pub p_state: TooltipTile,
    pub p_level: TooltipTile,
    pub fan_speed: TooltipTile,
    pub tx: TooltipTile,
    pub rx: TooltipTile,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TooltipTile {
    #[default(true)]
    pub enabled: bool,
    pub text: String,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            gpu_utilization: "GPU".into(),
            mem_utilization: "MEM USED".into(),
            mem_rw: "MEM R/W".into(),
            decoder_utilization: "DEC".into(),
            encoder_utilization: "ENC".into(),
            temperature: "TEMP".into(),
            power: "POWER".into(),
            p_state: "PSTATE".into(),
            p_level: "PLEVEL".into(),
            fan_speed: "FAN SPEED".into(),
            tx: "TX".into(),
            rx: "RX".into(),
        }
    }
}

impl From<&str> for TooltipTile {
    fn from(text: &str) -> Self {
        TooltipTile {
            text: text.to_string(),
            ..Default::default()
        }
    }
}

impl TooltipTile {
    pub fn get_text(&self) -> Option<&String> {
        if self.enabled {
            Some(&self.text)
        } else {
            None
        }
    }
}
