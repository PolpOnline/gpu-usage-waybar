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
        let hw_mon = gpu_handle.hw_monitors.first();

        let temps = hw_mon.map(HwMon::get_temps).unwrap_or_default();
        const TEMP_SENSOR_NAME: &str = "edge";
        let temp = temps
            .into_iter()
            .find(|(label, _)| label == TEMP_SENSOR_NAME)
            .and_then(|(_, temperature)| temperature.current);

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
            power: hw_mon.and_then(read_power),
            p_level: gpu_handle.get_power_force_performance_level().ok(),
            fan_speed: hw_mon.and_then(|hw_mon| fan_percentage(hw_mon).ok()),
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

/// Reads the current power draw.
///
/// The amdgpu driver may expose either `power1_input` (instantaneous power) or
/// `power1_average` (average power), depending on the ASIC, so fall back to the
/// average when no instantaneous reading is available.
fn read_power(hw_mon: &HwMon) -> Option<Power> {
    hw_mon
        .get_power_input()
        .or_else(|_| hw_mon.get_power_average())
        .ok()
        .map(|v| Power::new::<watt>(v as f32))
}

/// A minimal fake `cardN/device` sysfs tree modelled on a real AMD dGPU dump.
///
/// This allows exercising the AMD code path on machines without an AMD GPU.
#[cfg(test)]
struct MockSysFs {
    path: PathBuf,
}

#[cfg(test)]
impl MockSysFs {
    fn new(test_name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gpu-usage-waybar-{}-{test_name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn with_dgpu_data(test_name: &str, performance_level: &str) -> Self {
        let sysfs = Self::new(test_name);

        sysfs.write(
            "uevent",
            "DRIVER=amdgpu\nPCI_CLASS=30000\nPCI_ID=1002:7550\n\
             PCI_SUBSYS_ID=148C:2435\nPCI_SLOT_NAME=0000:09:00.0\n",
        );
        sysfs.write("gpu_busy_percent", "42");
        sysfs.write("mem_info_vram_used", "2147483648");
        sysfs.write("mem_info_vram_total", "8589934592");
        sysfs.write("power_dpm_force_performance_level", performance_level);

        // hwmon (numbers are in millidegrees/microwatts/RPM as exposed by the kernel)
        sysfs.write("hwmon/hwmon3/name", "amdgpu");
        sysfs.write("hwmon/hwmon3/temp1_label", "edge");
        sysfs.write("hwmon/hwmon3/temp1_input", "39000");
        sysfs.write("hwmon/hwmon3/temp2_label", "junction");
        sysfs.write("hwmon/hwmon3/temp2_input", "45000");
        sysfs.write("hwmon/hwmon3/temp3_label", "mem");
        sysfs.write("hwmon/hwmon3/temp3_input", "52000");
        sysfs.write("hwmon/hwmon3/power1_input", "120000000");
        sysfs.write("hwmon/hwmon3/fan1_input", "1500");
        sysfs.write("hwmon/hwmon3/fan1_max", "3000");

        sysfs
    }

    /// Emulate cards that only expose `power1_average` (most AMD dGPUs).
    fn with_average_power_only(self) -> Self {
        std::fs::remove_file(self.path.join("hwmon/hwmon3/power1_input")).unwrap();
        self.write("hwmon/hwmon3/power1_average", "95000000");

        self
    }
}

#[cfg(test)]
impl Drop for MockSysFs {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use amdgpu_sysfs::gpu_handle::PerformanceLevel;
    use uom::si::{
        f32::{Power, ThermodynamicTemperature},
        power::watt,
        thermodynamic_temperature::degree_celsius,
    };

    use super::*;
    use crate::formatter::State;

    fn compute(sysfs: &MockSysFs) -> GpuStatusData {
        let gpu_handle = GpuHandle::new_from_path(sysfs.path.clone()).unwrap();
        // `AmdGpuStatus` borrows its `AmdSysFS` for `'static` like `main` does.
        let amd_sys_fs: &'static AmdSysFS = Box::leak(Box::new(AmdSysFS { gpu_handle }));
        AmdGpuStatus::new(amd_sys_fs).unwrap().compute().unwrap()
    }

    #[test]
    fn test_compute_dgpu() {
        let sysfs = MockSysFs::with_dgpu_data("compute-dgpu", "auto");
        let data = compute(&sysfs);

        assert!(data.powered_on);
        assert!(data.has_running_processes);
        assert_eq!(data.gpu_utilization, Some(42));
        assert_eq!(data.compute_mem_usage(), Some(25));
        assert_eq!(
            data.temperature,
            Some(ThermodynamicTemperature::new::<degree_celsius>(39.0))
        );
        assert_eq!(data.power, Some(Power::new::<watt>(120.0)));
        assert_eq!(data.p_level, Some(PerformanceLevel::Auto));
        assert_eq!(data.fan_speed, Some(50));

        // Fields the AMD backend does not provide should stay `None`
        assert_eq!(data.mem_rw, None);
        assert_eq!(data.decoder_utilization, None);
        assert_eq!(data.encoder_utilization, None);
        assert!(data.p_state.is_none());
        assert_eq!(data.tx, None);
        assert_eq!(data.rx, None);
    }

    #[test]
    fn test_compute_formats_output() {
        let sysfs = MockSysFs::with_dgpu_data("format-output", "auto");
        let data = compute(&sysfs);

        let mut text = State::try_from_format("{gpu_utilization}%|{mem_utilization}%").unwrap();
        assert_eq!(data.get_text(&mut text), "42%|25%");

        let mut tooltip =
            State::try_from_format("{temperature:c}°C {power:w}W {p_level} {fan_speed}%").unwrap();
        assert_eq!(data.get_tooltip(&mut tooltip), "39°C 120W auto 50%");
    }

    #[test]
    fn test_compute_parses_new_performance_levels() {
        // New variants exposed by amdgpu-sysfs 0.22
        let sysfs = MockSysFs::with_dgpu_data("performance-level", "profile_peak");
        let data = compute(&sysfs);

        assert_eq!(data.p_level, Some(PerformanceLevel::ProfilePeak));

        let mut tooltip = State::try_from_format("{p_level}").unwrap();
        assert_eq!(data.get_tooltip(&mut tooltip), "profile_peak");
    }

    #[test]
    fn test_compute_falls_back_to_average_power() {
        let sysfs = MockSysFs::with_dgpu_data("average-power", "auto").with_average_power_only();
        let data = compute(&sysfs);

        assert_eq!(data.power, Some(Power::new::<watt>(95.0)));

        let mut tooltip = State::try_from_format("{power:w}W").unwrap();
        assert_eq!(data.get_tooltip(&mut tooltip), "95W");
    }

    #[test]
    fn test_compute_without_hwmon() {
        // e.g. a container/VM where hwmon is not exposed: must not panic and
        // must still report the metrics that come from the device directory.
        let sysfs = MockSysFs::new("no-hwmon");
        sysfs.write("uevent", "DRIVER=amdgpu\nPCI_CLASS=30000\n");
        sysfs.write("gpu_busy_percent", "7");
        sysfs.write("power_dpm_force_performance_level", "auto");

        let data = compute(&sysfs);

        assert_eq!(data.gpu_utilization, Some(7));
        assert_eq!(data.temperature, None);
        assert_eq!(data.power, None);
        assert_eq!(data.fan_speed, None);
        assert_eq!(data.p_level, Some(PerformanceLevel::Auto));
    }

    #[test]
    fn test_compute_without_edge_sensor() {
        let sysfs = MockSysFs::new("no-edge");
        sysfs.write("uevent", "DRIVER=amdgpu\nPCI_CLASS=30000\n");
        sysfs.write("hwmon/hwmon0/name", "amdgpu");
        sysfs.write("hwmon/hwmon0/temp1_label", "junction");
        sysfs.write("hwmon/hwmon0/temp1_input", "45000");

        let data = compute(&sysfs);

        assert_eq!(data.temperature, None);
    }
}
