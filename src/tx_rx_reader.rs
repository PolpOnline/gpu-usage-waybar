use color_eyre::eyre::{anyhow, Result};
use nvml_wrapper::enum_wrappers::device::PcieUtilCounter;
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static NVML_INSTANCE: OnceCell<Mutex<core::result::Result<Nvml, NvmlError>>> = OnceCell::new();

pub fn read_tx_rx() -> Result<TxRx> {
    let nvml = NVML_INSTANCE
        .get_or_init(|| {
            let nvml = Nvml::init();
            Mutex::new(nvml)
        })
        .lock()
        .unwrap();

    let nvml = nvml
        .as_ref()
        .map_err(|e| anyhow!("Failed to initialize NVML {}", e))?;

    let device = nvml.device_by_index(0)?;

    let tx = device.pcie_throughput(PcieUtilCounter::Send)? as f64 / 1000f64;
    let rx = device.pcie_throughput(PcieUtilCounter::Receive)? as f64 / 1000f64;

    Ok(TxRx { tx, rx })
}

pub struct TxRx {
    pub tx: f64,
    pub rx: f64,
}
