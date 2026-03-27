use std::ffi::OsString;

use color_eyre::eyre;
use procfs::process::ProcessesIter;

use crate::{
    drm::client::ClientManager,
    gpu_status::{GetFieldError, GpuStatus, fields::U8Field},
};

pub struct IntelGpuStatus {
    client_manager: ClientManager,
}

impl IntelGpuStatus {
    pub fn new(devnames: Box<[OsString]>) -> Self {
        Self {
            client_manager: ClientManager::new(devnames),
        }
    }

    fn compute_render_utilization(&self) -> f64 {
        self.client_manager
            .clients
            .iter()
            .map(|c| c.render_engine.utilization.unwrap_or_default())
            .sum()
    }

    fn compute_video_utilization(&self) -> f64 {
        self.client_manager
            .clients
            .iter()
            .map(|c| c.video_engine.utilization.unwrap_or_default())
            .sum()
    }
}

impl GpuStatus for IntelGpuStatus {
    fn update(&mut self, procs: ProcessesIter) -> eyre::Result<()> {
        self.client_manager.update(procs)?;
        Ok(())
    }

    fn get_u8_field(&self, field: U8Field) -> Result<u8, GetFieldError> {
        let render_utilization = self.compute_render_utilization();
        let video_utilization = self.compute_video_utilization();

        let decimal = match field {
            U8Field::GpuUtilization => render_utilization.max(video_utilization),
            U8Field::RenderUtilization => render_utilization,
            U8Field::VideoUtilization => video_utilization,
            _ => return Err(GetFieldError::BrandUnsupported),
        };

        Ok((decimal * 100.0).round() as u8)
    }
}
