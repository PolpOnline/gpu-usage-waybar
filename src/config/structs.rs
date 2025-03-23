use serde::Deserialize;
use smart_default::SmartDefault;

use crate::Args;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub general: Option<GeneralConfig>,
    pub text_config: Option<TextConfig>,
    pub tooltip_config: Option<TooltipConfig>,
}

impl ConfigFile {
    pub fn merge_args_into_config(&mut self, args: &Args) -> color_eyre::Result<()> {
        if args.text_no_memory {
            self.text_config = Some(TextConfig {
                show_memory: Some(false),
            });
        }

        if let Some(interval) = args.interval {
            match &mut self.general {
                Some(general_config) => {
                    general_config.interval = Some(interval);
                }
                None => {
                    self.general = Some(GeneralConfig {
                        interval: Some(interval),
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, SmartDefault)]
pub struct TextConfig {
    #[default(Some(true))]
    pub show_memory: Option<bool>,
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    #[default(Some(1000))]
    pub interval: Option<u64>,
}

#[derive(Deserialize)]
pub struct TooltipConfig {
    pub gpu_utilization: Option<TooltipTile>,
    pub mem_used: Option<TooltipTile>,
    pub mem_utilization: Option<TooltipTile>,
    pub decoder_utilization: Option<TooltipTile>,
    pub encoder_utilization: Option<TooltipTile>,
    pub temperature: Option<TooltipTile>,
    pub power: Option<TooltipTile>,
    pub performance_state: Option<TooltipTile>,
    pub performance_level: Option<TooltipTile>,
    pub fan_speed: Option<TooltipTile>,
    pub tx: Option<TooltipTile>,
    pub rx: Option<TooltipTile>,
}

#[derive(Deserialize)]
pub struct TooltipTile {
    pub enabled: bool,
    pub icon: String,
    pub text: String,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            gpu_utilization: Some(TooltipTile::new("".to_string(), "GPU".to_string())),
            mem_used: Some(TooltipTile::new("".to_string(), "MEM USED".to_string())),
            mem_utilization: Some(TooltipTile::new("".to_string(), "MEM R/W".to_string())),
            decoder_utilization: Some(TooltipTile::new("".to_string(), "DEC".to_string())),
            encoder_utilization: Some(TooltipTile::new("".to_string(), "ENC".to_string())),
            temperature: Some(TooltipTile::new("".to_string(), "TEMP".to_string())),
            power: Some(TooltipTile::new("".to_string(), "POWER".to_string())),
            performance_state: Some(TooltipTile::new("".to_string(), "PSTATE".to_string())),
            performance_level: Some(TooltipTile::new("".to_string(), "PLEVEL".to_string())),
            fan_speed: Some(TooltipTile::new("".to_string(), "FAN SPEED".to_string())),
            tx: Some(TooltipTile::new("".to_string(), "TX".to_string())),
            rx: Some(TooltipTile::new("".to_string(), "RX".to_string())),
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

impl ConfigFile {
    pub fn get_interval(&self) -> u64 {
        self.general
            .as_ref()
            .and_then(|cfg| cfg.interval)
            .unwrap_or_else(|| GeneralConfig::default().interval.unwrap_or_default())
    }

    pub fn get_text_show_memory(&self) -> bool {
        self.text_config
            .as_ref()
            .map(|cfg| cfg.show_memory.unwrap_or_default())
            .unwrap_or_else(|| TextConfig::default().show_memory.unwrap_or_default())
    }
}
