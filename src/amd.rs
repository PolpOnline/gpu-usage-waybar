use crate::gpu_status::{GpuStatus, GpuStatusData};
use amdgpu_sysfs::gpu_handle::GpuHandle;
use color_eyre::eyre::{eyre, Result};
use regex::Regex;
use std::path::PathBuf;

pub struct AmdGpuStatus {
    amd_sys_fs: &'static AmdSysFS,
}

impl AmdGpuStatus {
    pub fn new(amd_sys_fs: &'static AmdSysFS) -> Result<Self> {
        Ok(Self { amd_sys_fs })
    }
}

impl GpuStatus for AmdGpuStatus {
    fn compute(&self) -> Result<GpuStatusData> {
        let gpu_handle = &self.amd_sys_fs.gpu_handle;

        // TODO: add support for more metrics

        Ok(GpuStatusData {
            gpu_util: Some(gpu_handle.get_busy_percent()?),
            mem_used: Some(gpu_handle.get_used_vram()? as f64 / 1024f64 / 1024f64), // convert to MiB from B
            mem_total: Some(gpu_handle.get_total_vram()? as f64 / 1024f64 / 1024f64),
            mem_util: None,
            dec_util: None,
            enc_util: None,
            temp: None,
            power: None,
            p_state: None,
            fan_speed: None,
            tx: None,
            rx: None,
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

        for entry in drm_dir.read_dir()? {
            let entry = entry?;
            let mut path = entry.path();

            if path.is_dir() {
                let drm_device = path
                    .file_name()
                    .ok_or(eyre!("Path terminates in \"..\" or \".\""))?
                    .to_str()
                    .ok_or(eyre!("Path isn't a valid UTF-8"))?;

                let re = Regex::new(r"^card1$")?;

                if re.is_match(drm_device) {
                    path.push(PathBuf::from("device"));
                    drm_gpus.push(path);
                }
            }
        }

        Ok(drm_gpus)
    }
}
