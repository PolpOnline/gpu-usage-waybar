use std::path::PathBuf;

use amdgpu_sysfs::{gpu_handle::GpuHandle, hw_mon::HwMon};
use color_eyre::eyre::{Result, eyre};
use regex::Regex;
use uom::si::{
    f32::Information, f32::Power, information::byte, power::watt,
    thermodynamic_temperature::degree_celsius,
};

use crate::gpu_status::{GpuStatus, GpuStatusData, Temperature};

pub struct AmdGpuStatus {
    amd_sys_fs: &'static AmdSysFS,
}

impl AmdGpuStatus {
    pub const fn new(amd_sys_fs: &'static AmdSysFS) -> Result<Self> {
        Ok(Self { amd_sys_fs })
    }
}

impl GpuStatus for AmdGpuStatus {
    fn compute(&self) -> Result<GpuStatusData> {
        let gpu_handle = &self.amd_sys_fs.gpu_handle;
        let hw_mon = &gpu_handle.hw_monitors[0];

        let temps = hw_mon.get_temps();
        const TEMP_SENSOR_NAME: &str = "edge";
        let temp = temps
            .iter()
            .find(|t| t.0 == TEMP_SENSOR_NAME)
            .ok_or(eyre!(format!(
                "No \"{}\" temperature sensor found",
                TEMP_SENSOR_NAME
            )))?
            .1
            .current;

        Ok(GpuStatusData {
            powered_on: true,
            has_running_processes: true, /* TODO: temporarily set to true until AMD GPU process
                                          * detection is implemented */
            gpu_utilization: gpu_handle.get_busy_percent().ok(),
            mem_used: gpu_handle
                .get_used_vram()
                .ok()
                .map(|v| Information::new::<byte>(v as f32)),
            mem_total: gpu_handle
                .get_total_vram()
                .ok()
                .map(|v| Information::new::<byte>(v as f32)),
            temperature: temp.map(Temperature::new::<degree_celsius>),
            power: hw_mon
                .get_power_input()
                .ok()
                .map(|v| Power::new::<watt>(v as f32)),
            p_level: gpu_handle.get_power_force_performance_level().ok(),
            fan_speed: fan_percentage(hw_mon).ok(),
            ..Default::default()
        })
    }
}

pub struct AmdSysFS {
    gpu_handle: GpuHandle,
}

impl AmdSysFS {
    pub fn init() -> Result<Self> {
        let drm_gpus = Self::get_drm_gpus()?;

        if drm_gpus.is_empty() {
            return Err(eyre!("No AMD GPU found"));
        }

        let gpu_handle = GpuHandle::new_from_path(drm_gpus[0].clone())?;

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

fn fan_percentage(hw_mon: &HwMon) -> Result<u8, amdgpu_sysfs::error::Error> {
    let current_rpm = hw_mon.get_fan_current()? as f32;
    let max_rpm = hw_mon.get_fan_max()? as f32;

    Ok((current_rpm / max_rpm * 100.0).round().clamp(0.0, 100.0) as u8)
}
