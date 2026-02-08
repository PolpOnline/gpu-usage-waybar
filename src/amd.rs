use std::path::PathBuf;

use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::{Result, eyre};
use regex::Regex;
use uom::si::{
    f32::Information, f32::Power, information::byte, power::watt,
    thermodynamic_temperature::degree_celsius,
};

use crate::gpu_status::{GetFieldError, fields::*};
use crate::gpu_status::{GpuStatus, Temperature};

pub struct AmdGpuStatus {
    amd_sys_fs: &'static AmdSysFS,
}

impl AmdGpuStatus {
    pub const fn new(amd_sys_fs: &'static AmdSysFS) -> Self {
        Self { amd_sys_fs }
    }
    fn fan_percentage(&self) -> Result<u8, amdgpu_sysfs::error::Error> {
        let handle = &self.amd_sys_fs.gpu_handle;
        let hw_mon = &handle.hw_monitors[0];
        let current_rpm = hw_mon.get_fan_current()? as f32;
        let max_rpm = hw_mon.get_fan_max()? as f32;

        Ok((current_rpm / max_rpm * 100.0).round().clamp(0.0, 100.0) as u8)
    }
}

impl GpuStatus for AmdGpuStatus {
    fn get_u8_field(&self, field: U8Field) -> Result<u8, GetFieldError> {
        let handle = &self.amd_sys_fs.gpu_handle;
        let maybe_val = match field {
            U8Field::GpuUtilization => handle.get_busy_percent().ok(),
            U8Field::FanSpeed => self.fan_percentage().ok(),
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        maybe_val.ok_or(GetFieldError::Unavailable)
    }

    fn get_mem_field(&self, field: MemField) -> Result<Information, GetFieldError> {
        let handle = &self.amd_sys_fs.gpu_handle;
        let maybe_val = match field {
            MemField::MemUsed => handle.get_used_vram(),
            MemField::MemTotal => handle.get_total_vram(),
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        maybe_val
            .map(|v| Information::new::<byte>(v as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
        let handle = &self.amd_sys_fs.gpu_handle;
        let hw_mon = &handle.hw_monitors[0];
        let temps = hw_mon.get_temps();

        const TEMP_SENSOR_NAME: &str = "edge";
        let temp = temps
            .iter()
            .find(|t| t.0 == TEMP_SENSOR_NAME)
            .ok_or(eyre!(format!(
                "No \"{}\" temperature sensor found",
                TEMP_SENSOR_NAME
            )))
            .map_err(|_| GetFieldError::Unavailable)?;
        let temp = temp.1.current.ok_or(GetFieldError::Unavailable)?;

        Ok(Temperature::new::<degree_celsius>(temp))
    }

    fn get_power(&self) -> Result<Power, GetFieldError> {
        let handle = &self.amd_sys_fs.gpu_handle;
        let hw_mon = &handle.hw_monitors[0];
        hw_mon
            .get_power_input()
            .map(|v| Power::new::<watt>(v as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_plevel(&self) -> Result<PerformanceLevel, GetFieldError> {
        let handle = &self.amd_sys_fs.gpu_handle;
        handle
            .get_power_force_performance_level()
            .map_err(|_| GetFieldError::Unavailable)
    }
}

type AmdGpuHandle = amdgpu_sysfs::gpu_handle::GpuHandle;

pub struct AmdSysFS {
    gpu_handle: AmdGpuHandle,
}

impl AmdSysFS {
    pub fn init() -> Result<Self> {
        let drm_gpus = Self::get_drm_gpus()?;

        if drm_gpus.is_empty() {
            return Err(eyre!("No AMD GPU found"));
        }

        let gpu_handle = AmdGpuHandle::new_from_path(drm_gpus[0].clone())?;

        Ok(Self { gpu_handle })
    }

    fn get_drm_gpus() -> Result<Vec<PathBuf>> {
        let drm_dir = PathBuf::from("/sys/class/drm");
        let mut drm_gpus = Vec::new();

        let card_regex = Regex::new(r"^card[0-9]*$")?;

        for entry in drm_dir.read_dir()? {
            let entry = entry?;
            let mut path = entry.path();

            if path.is_dir() {
                let drm_device = path
                    .file_name()
                    .ok_or(eyre!("Path terminates in \"..\" or \".\""))?
                    .to_str()
                    .ok_or(eyre!("Path isn't a valid UTF-8"))?;

                if card_regex.is_match(drm_device) {
                    path.push(PathBuf::from("device"));
                    drm_gpus.push(path);
                }
            }
        }

        Ok(drm_gpus)
    }
}
