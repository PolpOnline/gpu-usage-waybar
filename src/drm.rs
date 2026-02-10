use std::ffi::OsString;

use color_eyre::eyre::{self, OptionExt};

/// A wrapper around [udev::Device].
///
/// A [DrmDevice] is the parent of DRM leaf nodes,
/// usually being `card0` and `renderD128`.
#[derive(Debug)]
pub struct DrmDevice(udev::Device);

impl DrmDevice {
    fn get_vendor_id_str(&self) -> eyre::Result<&str> {
        let pci_id = self
            .0
            .property_value("PCI_ID")
            .ok_or_eyre("Cannot find PCI_ID for device")?
            .to_str()
            .unwrap();
        Ok(&pci_id[0..4])
    }

    pub fn get_vendor_name(&self) -> eyre::Result<OsString> {
        let hwdb = udev::Hwdb::new()?;
        let modalias = format!("pci:v0000{}*", self.get_vendor_id_str()?);
        let vendor_name = hwdb
            .query_one(modalias.as_str(), "ID_VENDOR_FROM_DATABASE")
            .ok_or_eyre("No vendor name result exits in database")?;
        Ok(vendor_name.to_owned())
    }
}

pub fn scan_drm_devices() -> eyre::Result<Vec<DrmDevice>> {
    // construct an enumerator that iterates through DRM leaf nodes
    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("drm").unwrap();
    enumerator.match_property("DEVNAME", "/dev/dri/*").unwrap();

    let mut drm_devices = Vec::new();
    for dev in enumerator.scan_devices()? {
        let parent = dev.parent().expect("DRM device must have a parent");

        if !drm_devices
            .iter()
            .any(|drm_dev: &DrmDevice| drm_dev.0.sysname() == parent.sysname())
        {
            drm_devices.push(DrmDevice(parent));
        }
    }

    Ok(drm_devices)
}

#[test]
fn test_scan_drm_devices() {
    let devs = scan_drm_devices().unwrap();
    dbg!(devs);
}
