use std::ffi::OsString;

use color_eyre::eyre::{self, OptionExt};
use udev::Hwdb;

/// A wrapper around [udev::Device].
///
/// A [DrmDevice] is the parent of DRM leaf nodes.
/// The leaf nodes (its children) are usually `cardN` and `renderDN`.
#[derive(Debug)]
pub struct DrmDevice {
    pub device: udev::Device,
    pub children: Vec<udev::Device>,
    pci_id: PciId,
}

impl DrmDevice {
    pub fn new(
        device: udev::Device,
        children: Vec<udev::Device>,
    ) -> Result<Self, NotPciDeviceError> {
        let pci_id = PciId::from_device(&device).ok_or(NotPciDeviceError(device.clone()))?;

        Ok(Self {
            device,
            children,
            pci_id,
        })
    }

    /// Return the card index `N` if a child with sysname `cardN` is found.
    pub fn get_dri_card_index(&self) -> Option<u8> {
        let maybe_index_str = self
            .children
            .iter()
            .find_map(|dev| dev.sysname().to_str().unwrap().strip_prefix("card"));

        maybe_index_str.map(|idx| idx.parse().unwrap())
    }

    pub fn get_model_name(&self, hwdb: &Hwdb) -> eyre::Result<OsString> {
        let modalias = format!(
            "pci:v{:08X}d{:08X}*",
            self.pci_id.vendor_id, self.pci_id.device_id,
        );

        let model_name = hwdb
            .query_one(modalias.as_str(), "ID_MODEL_FROM_DATABASE")
            .ok_or_eyre("No model name result exists in database")?;

        Ok(model_name.to_owned())
    }

    pub fn get_vendor_name(&self, hwdb: &Hwdb) -> eyre::Result<OsString> {
        let modalias = format!("pci:v{:08X}*", self.pci_id.vendor_id);
        let vendor_name = hwdb
            .query_one(modalias.as_str(), "ID_VENDOR_FROM_DATABASE")
            .ok_or_eyre("No vendor name result exists in database")?;
        Ok(vendor_name.to_owned())
    }
}

/// Scan DRM devices and sort them by card index.
pub fn scan_drm_devices() -> eyre::Result<Vec<DrmDevice>> {
    // construct an enumerator that iterates through DRM leaf nodes
    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("drm").unwrap();
    enumerator.match_property("DEVNAME", "/dev/dri/*").unwrap();

    let mut drm_devices: Vec<DrmDevice> = Vec::new();
    for dev in enumerator.scan_devices()? {
        let parent = dev.parent().expect("DRM device must have a parent");

        if let Some(drm_device) = drm_devices
            .iter_mut()
            .find(|drm_dev| drm_dev.device.sysname() == parent.sysname())
        {
            drm_device.children.push(dev);
        } else {
            drm_devices.push(DrmDevice::new(parent, vec![dev]).unwrap());
        }
    }

    // sort by card index
    drm_devices.sort_by_key(|dev| dev.get_dri_card_index().unwrap_or(u8::MAX));

    Ok(drm_devices)
}

#[derive(Debug)]
pub struct NotPciDeviceError(udev::Device);
impl std::fmt::Display for NotPciDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} is not a PCI device", self.0)
    }
}
impl std::error::Error for NotPciDeviceError {}

#[derive(Debug, Clone, Copy)]
struct PciId {
    pub vendor_id: u16,
    pub device_id: u16,
}

impl PciId {
    fn from_device(dev: &udev::Device) -> Option<Self> {
        // PCI ID is in format XXXX:XXXX
        let pci_id_osstr = dev.property_value("PCI_ID")?;
        let pci_id_str = pci_id_osstr.to_str().unwrap();
        let (vendor_str, device_str) = pci_id_str.split_once(':').unwrap();

        let vendor_id = u16::from_str_radix(vendor_str, 16).unwrap();
        let device_id = u16::from_str_radix(device_str, 16).unwrap();

        Some(Self {
            vendor_id,
            device_id,
        })
    }
}
