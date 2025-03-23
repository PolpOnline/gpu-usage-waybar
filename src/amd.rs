use std::path::PathBuf;

use amdgpu_sysfs::gpu_handle::GpuHandle;
use color_eyre::eyre::{eyre, Result};
use regex::Regex;

use crate::gpu_status::{GpuStatus, GpuStatusData};

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
            gpu_util: gpu_handle.get_busy_percent().ok(),
            mem_used: gpu_handle
                .get_used_vram()
                .ok()
                .map(|v| v as f64 / 1024f64 / 1024f64), // convert to MiB from B
            mem_total: gpu_handle
                .get_total_vram()
                .ok()
                .map(|v| v as f64 / 1024f64 / 1024f64),
            temp: temp.map(|v| v.round() as u8),
            power: hw_mon.get_power_input().ok(),
            p_level: gpu_handle.get_power_force_performance_level().ok(),
            fan_speed: hw_mon.get_fan_current().ok().map(|v| v as u8),
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
