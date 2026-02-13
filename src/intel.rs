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
        let clients = ClientManager::new(devnames);
        Self {
            client_manager: clients,
        }
    }

    fn compute_utilization(&self) -> Option<f64> {
        // TODO: mix multiple engines
        let mut utilization = 0.0;

        for client in &self.client_manager.clients {
            let client_utilization = client.render_engine.utilization.unwrap_or_default();
            utilization += client_utilization;
        }

        Some(utilization)
    }
}

impl GpuStatus for IntelGpuStatus {
    fn update(&mut self, procs: ProcessesIter) -> eyre::Result<()> {
        self.client_manager.update(procs);
        Ok(())
    }

    fn get_u8_field(&self, field: U8Field) -> Result<u8, GetFieldError> {
        match field {
            U8Field::GpuUtilization => {
                let utilization = self.compute_utilization().ok_or(GetFieldError::NotReady)?;
                Ok((utilization * 100.0) as u8)
            }
            _ => Err(GetFieldError::BrandUnsupported),
        }
    }
}
