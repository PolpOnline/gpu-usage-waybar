use crate::gpu_status::{GpuStatus, GpuStatusData};
use crate::utils::is_program_in_path;
use color_eyre::eyre::{eyre, Result};

const AMDGPU_TOP_EXECUTABLE: &str = "amdgpu_top";

pub struct AmdGpuStatus {}

impl AmdGpuStatus {
    pub fn new() -> Result<Self> {
        if is_program_in_path(AMDGPU_TOP_EXECUTABLE) {
            return Ok(Self {});
        }

        Err(eyre!("{} needs to be installed", AMDGPU_TOP_EXECUTABLE))
    }
}

impl GpuStatus for AmdGpuStatus {
    fn compute(&self) -> Result<GpuStatusData> {
        Ok(GpuStatusData::default())
    }
}
