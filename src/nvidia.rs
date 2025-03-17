use crate::gpu_status::{GpuStatus, GpuStatusData, PState};
use color_eyre::eyre::Result;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, PerformanceState, TemperatureSensor};
use nvml_wrapper::{Device, Nvml};
use std::fs;

pub struct NvidiaGpuStatus<'a> {
    device: Device<'a>,
    bus_id: String
}

impl NvidiaGpuStatus<'_> {
    pub fn new(instance: &'static Nvml) -> Result<Self> {
        let device = instance.device_by_index(0)?;

        // Query PCI info just once
        // NVML returns a PCI domain up to 0xffffffff; need to truncate
        // to match sysfs
        let bus_id = device.pci_info()?.bus_id.chars().skip(4).collect();

        Ok(Self { device, bus_id })
    }
}

fn is_powered_on(bus_id: &str) -> Result<bool, std::io::Error> {
    let path = format!("/sys/bus/pci/devices/{}/power/runtime_status", bus_id);
    let status = fs::read_to_string(path)?.trim().to_string();
    let powered_on = status == "active";
    Ok(powered_on)
}

impl GpuStatus for NvidiaGpuStatus<'_> {
    fn compute(&self) -> Result<GpuStatusData> {
        // NVML queries inadvertently wake the NVIDIA card
        // Use sysfs to check power status first
        let powered_on = is_powered_on(&self.bus_id)?;
        let gpu_status = if !powered_on {
            GpuStatusData {
              powered_on: false,
              ..Default::default()
            }
        } else {
          let device = &self.device;

          let utilization_rates = device.utilization_rates().ok();
          let memory_info_in_bytes = device.memory_info().ok();

          GpuStatusData {
              powered_on: true,
              gpu_util: utilization_rates.clone().map(|u| u.gpu as u8),
              mem_used: memory_info_in_bytes
                  .clone()
                  .map(|m| m.used as f64 / 1024f64 / 1024f64), // convert to MiB from B
              mem_total: memory_info_in_bytes.map(|m| m.total as f64 / 1024f64 / 1024f64),
              mem_util: utilization_rates.map(|u| u.memory as u8),
              dec_util: device
                  .decoder_utilization()
                  .ok()
                  .map(|u| u.utilization as u8),
              enc_util: device
                  .encoder_utilization()
                  .ok()
                  .map(|u| u.utilization as u8),
              temp: device
                  .temperature(TemperatureSensor::Gpu)
                  .ok()
                  .map(|t| t as u8),
              power: device.power_usage().ok().map(|p| p as f64 / 1000f64), // convert to W from mW
              p_state: device.performance_state().ok().map(|p| p.into()),
              fan_speed: device.fan_speed(0u32).ok().map(|f| f as u8),
              tx: device
                  .pcie_throughput(PcieUtilCounter::Send)
                  .ok()
                  .map(|t| t as f64 / 1000f64), // convert to MiB/s from KiB/s
              rx: device
                  .pcie_throughput(PcieUtilCounter::Receive)
                  .ok()
                  .map(|t| t as f64 / 1000f64),
              ..Default::default()
          }
        };

        Ok(gpu_status)
    }
}

impl From<PerformanceState> for PState {
    fn from(value: PerformanceState) -> Self {
        match value {
            PerformanceState::Zero => PState::P0,
            PerformanceState::One => PState::P1,
            PerformanceState::Two => PState::P2,
            PerformanceState::Three => PState::P3,
            PerformanceState::Four => PState::P4,
            PerformanceState::Five => PState::P5,
            PerformanceState::Six => PState::P6,
            PerformanceState::Seven => PState::P7,
            PerformanceState::Eight => PState::P8,
            PerformanceState::Nine => PState::P9,
            PerformanceState::Ten => PState::P10,
            PerformanceState::Eleven => PState::P11,
            PerformanceState::Twelve => PState::P12,
            PerformanceState::Thirteen => PState::P13,
            PerformanceState::Fourteen => PState::P14,
            PerformanceState::Fifteen => PState::P15,
            PerformanceState::Unknown => PState::Unknown,
        }
    }
}
