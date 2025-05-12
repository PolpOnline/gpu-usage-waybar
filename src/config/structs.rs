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
            self.text.show_memory = false;
        }

        if let Some(interval) = args.interval {
            self.general.interval = interval;
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

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TooltipConfig {
    pub gpu_utilization: ConfigGpuUtilization,
    pub mem_used: ConfigMemUsed,
    pub mem_rw: ConfigMemRW,
    pub decoder_utilization: ConfigDecoderUtilization,
    pub encoder_utilization: ConfigEncoderUtilization,
    pub temperature: ConfigTemperature,
    pub power: ConfigPower,
    pub p_state: ConfigPerformanceState,
    pub p_level: ConfigPerformanceLevel,
    pub fan_speed: ConfigFanSpeed,
    pub tx: ConfigTx,
    pub rx: ConfigRx,
}

macro_rules! generate_icon_text_struct {
    ($name:ident, $default_text:expr) => {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[serde(default)]
        pub struct $name {
            pub enabled: bool,
            pub text: String,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    enabled: true,
                    text: $default_text.to_string(),
                }
            }
        }

        impl $name {
            pub fn get_text(&self) -> Option<&String> {
                if self.enabled { Some(&self.text) } else { None }
            }
        }
    };
}

generate_icon_text_struct!(ConfigGpuUtilization, "GPU");
generate_icon_text_struct!(ConfigMemUsed, "MEM USED");
generate_icon_text_struct!(ConfigMemRW, "MEM R/W");
generate_icon_text_struct!(ConfigDecoderUtilization, "DEC");
generate_icon_text_struct!(ConfigEncoderUtilization, "ENC");
generate_icon_text_struct!(ConfigTemperature, "TEMP");
generate_icon_text_struct!(ConfigPower, "POWER");
generate_icon_text_struct!(ConfigPerformanceState, "PSTATE");
generate_icon_text_struct!(ConfigPerformanceLevel, "PLEVEL");
generate_icon_text_struct!(ConfigFanSpeed, "FAN SPEED");
generate_icon_text_struct!(ConfigTx, "TX");
generate_icon_text_struct!(ConfigRx, "RX");
