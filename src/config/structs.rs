use crate::Args;
use serde::Deserialize;
use smart_default::SmartDefault;

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

#[derive(Deserialize, Default)]
pub struct TooltipConfig {
    pub gpu_utilization: Option<GpuUtilization>,
    pub mem_used: Option<MemUsed>,
    pub mem_utilization: Option<MemUtilization>,
    pub decoder_utilization: Option<DecoderUtilization>,
    pub encoder_utilization: Option<EncoderUtilization>,
    pub temperature: Option<Temperature>,
    pub power: Option<Power>,
    pub performance_state: Option<PerformanceState>,
    pub performance_level: Option<PerformanceLevel>,
    pub fan_speed: Option<FanSpeed>,
    pub tx: Option<Tx>,
    pub rx: Option<Rx>,
}

macro_rules! generate_icon_text_struct {
    ($name:ident, $default_icon:expr, $default_text:expr) => {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            pub enabled: bool,
            pub icon: Option<String>,
            pub text: Option<String>,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    enabled: true,
                    icon: Some($default_icon.to_string()),
                    text: Some($default_text.to_string()),
                }
            }
        }
    };
}

generate_icon_text_struct!(GpuUtilization, "", "GPU");
generate_icon_text_struct!(MemUsed, "", "MEM USED");
generate_icon_text_struct!(MemUtilization, "", "MEM R/W");
generate_icon_text_struct!(DecoderUtilization, "", "DEC");
generate_icon_text_struct!(EncoderUtilization, "", "ENC");
generate_icon_text_struct!(Temperature, "", "TEMP");
generate_icon_text_struct!(Power, "", "POWER");
generate_icon_text_struct!(PerformanceState, "", "PSTATE");
generate_icon_text_struct!(PerformanceLevel, "", "PLEVEL");
generate_icon_text_struct!(FanSpeed, "", "FAN SPEED");
generate_icon_text_struct!(Tx, "", "TX");
generate_icon_text_struct!(Rx, "", "RX");

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
