use std::ffi::OsString;

use color_eyre::eyre::{self, OptionExt};

/// A wrapper around [udev::Device].
///
/// A [DrmDevice] is the parent of DRM leaf nodes,
/// usually being `card0` and `renderD128`.
#[derive(Debug)]
pub struct DrmDevice {
    device: udev::Device,
    children: Vec<udev::Device>,
}

impl DrmDevice {
    pub fn get_dri_card_index(&self) -> Option<u8> {
        let maybe_index_str = self
            .children
            .iter()
            .find_map(|dev| dev.sysname().to_str().unwrap().strip_prefix("card"));

        maybe_index_str.map(|idx| idx.parse().unwrap())
    }

    pub fn get_vendor_name(&self) -> eyre::Result<OsString> {
        let hwdb = udev::Hwdb::new()?;
        let modalias = format!("pci:v0000{}*", self.get_vendor_id_str()?);
        let vendor_name = hwdb
            .query_one(modalias.as_str(), "ID_VENDOR_FROM_DATABASE")
            .ok_or_eyre("No vendor name result exits in database")?;
        Ok(vendor_name.to_owned())
    }

    fn get_vendor_id_str(&self) -> eyre::Result<&str> {
        let pci_id = self
            .device
            .property_value("PCI_ID")
            .ok_or_eyre("Cannot find PCI_ID for device")?
            .to_str()
            .unwrap();
        Ok(&pci_id[0..4])
    }
}

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
            drm_devices.push(DrmDevice {
                device: parent,
                children: vec![dev],
            });
        }
    }

    sort_by_dri_card_index(&mut drm_devices);

    Ok(drm_devices)
}

fn sort_by_dri_card_index(devs: &mut [DrmDevice]) {
    devs.sort_by_key(|dev| dev.get_dri_card_index().unwrap_or(u8::MAX));
}

#[test]
fn test_scan_drm_devices() {
    let devs = scan_drm_devices().unwrap();
    dbg!(devs);
}
