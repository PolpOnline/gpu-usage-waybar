use std::path::PathBuf;

use amdgpu_sysfs::gpu_handle::PerformanceLevel;
use color_eyre::eyre::{Result, eyre};
use uom::si::{
    f32::Information, f32::Power, information::byte, power::watt,
    thermodynamic_temperature::degree_celsius,
};

use crate::gpu_status::{GetFieldError, fields::*};
use crate::gpu_status::{GpuStatus, Temperature};

type AmdGpuHandle = amdgpu_sysfs::gpu_handle::GpuHandle;

pub struct AmdGpuStatus {
    handle: AmdGpuHandle,
}

impl AmdGpuStatus {
    pub fn new(sysfs_path: PathBuf) -> Result<Self, amdgpu_sysfs::error::Error> {
        let handle = AmdGpuHandle::new_from_path(sysfs_path)?;
        Ok(Self { handle })
    }

    fn fan_percentage(&self) -> Result<u8, amdgpu_sysfs::error::Error> {
        let hw_mon = &self.handle.hw_monitors[0];
        let current_rpm = hw_mon.get_fan_current()? as f32;
        let max_rpm = hw_mon.get_fan_max()? as f32;

        Ok((current_rpm / max_rpm * 100.0).round().clamp(0.0, 100.0) as u8)
    }
}

impl GpuStatus for AmdGpuStatus {
    fn get_u8_field(&self, field: U8Field) -> Result<u8, GetFieldError> {
        let maybe_val = match field {
            U8Field::GpuUtilization => self.handle.get_busy_percent().ok(),
            U8Field::FanSpeed => self.fan_percentage().ok(),
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        maybe_val.ok_or(GetFieldError::Unavailable)
    }

    fn get_mem_field(&self, field: MemField) -> Result<Information, GetFieldError> {
        let maybe_val = match field {
            MemField::MemUsed => self.handle.get_used_vram(),
            MemField::MemTotal => self.handle.get_total_vram(),
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        maybe_val
            .map(|v| Information::new::<byte>(v as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_temperature(&self) -> Result<Temperature, GetFieldError> {
        let hw_mon = &self.handle.hw_monitors[0];
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
        let hw_mon = &self.handle.hw_monitors[0];
        hw_mon
            .get_power_input()
            .map(|v| Power::new::<watt>(v as f32))
            .map_err(|_| GetFieldError::Unavailable)
    }

    fn get_plevel(&self) -> Result<PerformanceLevel, GetFieldError> {
        self.handle
            .get_power_force_performance_level()
            .map_err(|_| GetFieldError::Unavailable)
    }
}
