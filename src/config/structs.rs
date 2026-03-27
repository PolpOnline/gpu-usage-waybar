use color_eyre::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{
    Args,
    formatter::{self, FormatSegments},
    gpu_status::{GpuHandle, fields::Field},
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
            self.text.format = Format::new(text_format.to_owned());
        }
        if let Some(ref tooltip_format) = args.tooltip_format {
            self.tooltip.format = Format::new(tooltip_format.to_owned());
        }

        Ok(())
    }
}

#[derive(Deserialize, SmartDefault)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TextConfig {
    pub format: Format,
}

impl AssembleAvailables for TextConfig {
    /// Set `self.format` to "{gpu_utilization}%|{mem_utilization}%"
    /// if [Field::MemUtilization] in `handle` is available. Otherwise,
    /// set `self.format` to "{gpu_utilization}%".
    fn assemble_availables(&mut self, handle: &GpuHandle) {
        let mut result = "{gpu_utilization}%".to_string();
        if handle.is_field_available(Field::MemUtilization) {
            result.push_str("|{mem_utilization}%");
        }

        self.format = Format::new(result);
    }
}

// TODO: rearrange
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
    pub format: Format,
}

impl AssembleAvailables for TooltipConfig {
    /// Assemble available lines in the tooltip default format.
    ///
    /// # Note
    ///
    /// The line is only added if **all** fields in the line are available.
    fn assemble_availables(&mut self, handle: &GpuHandle) {
        const DEFAULT_FORMAT: &str = r"GPU: {gpu_utilization}%
RENDER: {render_utilization}%
VIDEO: {video_utilization}%
MEM USED: {mem_used:MiB.0}/{mem_total:MiB} MiB ({mem_utilization}%)
MEM R/W: {mem_rw}%
DEC: {decoder_utilization}%
ENC: {encoder_utilization}%
TEMP: {temperature:c}°C
POWER: {power:w}W
PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx:MiB.3} MiB/s
RX: {rx:MiB.3} MiB/s";

        let mut result = String::new();
        let re = formatter::get_regex();

        for line in DEFAULT_FORMAT.split_inclusive('\n') {
            // Check if ALL field string is invalid
            let is_all_available = re.captures_iter(line).all(|caps| {
                let format_segments = FormatSegments::from_caps_unchecked(&caps);
                // unwrapping from DEFAULT_FORMAT should be safe
                let field = Field::try_from(format_segments).unwrap();
                handle.is_field_available(field)
            });

            if is_all_available {
                result.push_str(line);
            }
        }

        self.format = Format::new(result);
    }
}

#[derive(Deserialize, SmartDefault)]
pub struct Format(pub Option<String>);

impl Format {
    pub fn new(s: String) -> Self {
        Self(Some(s))
    }

    pub fn is_set(&self) -> bool {
        self.0.is_some()
    }
}

pub trait AssembleAvailables {
    fn assemble_availables(&mut self, handle: &GpuHandle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gpu_status::{GetFieldError, GpuHandle, GpuStatus, fields::MemField},
        nvidia::PState,
    };
    use color_eyre::eyre::Result;
    use uom::si::{f32::Information, information::mebibyte};

    #[test]
    fn tooltip_assemble_availabes() {
        struct Data;
        impl GpuStatus for Data {
            fn get_pstate(&self) -> Result<PState, GetFieldError> {
                Ok(PState::P0)
            }
            fn get_mem_field(
                &self,
                field: crate::gpu_status::fields::MemField,
            ) -> Result<Information, GetFieldError> {
                match field {
                    MemField::Tx => Ok(Information::new::<mebibyte>(5.0)),
                    MemField::Rx => Ok(Information::new::<mebibyte>(6.0)),
                    _ => Err(GetFieldError::BrandUnsupported),
                }
            }
        }
        let handle = GpuHandle::new(Box::new(Data));

        let mut config = TooltipConfig {
            format: Format(None),
        };
        config.assemble_availables(&handle);

        assert_eq!(
            config.format.0.unwrap(),
            r"PSTATE: {p_state}
TX: {tx:MiB.3} MiB/s
RX: {rx:MiB.3} MiB/s"
        );
    }

    /// Test that lines with multiple placeholders are dropped if any of them
    /// have no value.
    #[test]
    fn tooltip_assemble_availables_multiple_placeholders() {
        struct Data1;
        impl GpuStatus for Data1 {
            fn get_mem_field(&self, field: MemField) -> Result<Information, GetFieldError> {
                match field {
                    MemField::MemUsed => Ok(Information::new::<mebibyte>(50.0)),
                    _ => Err(GetFieldError::BrandUnsupported),
                }
            }
        }
        let handle = GpuHandle::new(Box::new(Data1));

        let mut config = TooltipConfig {
            format: Format(None),
        };

        config.assemble_availables(&handle);
        // All lines should be dropped, including
        // "MEM USED: {mem_used:MiB.0}/{mem_total:MiB} MiB ({mem_utilization}%)"
        // because mem_total and mem_utilization are Err.
        assert_eq!(config.format.0.unwrap(), "".to_string());

        struct Data2;
        impl GpuStatus for Data2 {
            fn get_mem_field(&self, field: MemField) -> Result<Information, GetFieldError> {
                match field {
                    MemField::MemUsed => Ok(Information::new::<mebibyte>(50.0)),
                    MemField::MemTotal => Ok(Information::new::<mebibyte>(50.0)),
                    _ => Err(GetFieldError::BrandUnsupported),
                }
            }
        }
        let handle = GpuHandle::new(Box::new(Data2));

        let mut config = TooltipConfig {
            format: Format(None),
        };

        config.assemble_availables(&handle);
        // "MEM USED: {mem_used:MiB.0}/{mem_total:MiB} MiB ({mem_utilization}%)"
        // should retain becuase all fields here are available.
        assert_eq!(
            config.format.0.unwrap(),
            "MEM USED: {mem_used:MiB.0}/{mem_total:MiB} MiB ({mem_utilization}%)\n".to_string()
        );
    }

    #[test]
    fn text_assemble_availables() {
        struct Data1;
        impl GpuStatus for Data1 {} // No mem_utilization
        let handle = GpuHandle::new(Box::new(Data1));

        let mut config = TextConfig {
            format: Format(None),
        };

        config.assemble_availables(&handle);
        assert_eq!(config.format.0.unwrap(), "{gpu_utilization}%".to_string());

        struct Data2;
        impl GpuStatus for Data2 {
            fn get_mem_field(&self, _field: MemField) -> Result<Information, GetFieldError> {
                Ok(Information::new::<mebibyte>(100.0))
            }
        }
        let handle = GpuHandle::new(Box::new(Data2));

        let mut config = TextConfig {
            format: Format(None),
        };

        config.assemble_availables(&handle);
        assert_eq!(
            config.format.0.unwrap(),
            "{gpu_utilization}%|{mem_utilization}%".to_string()
        )
    }
}
