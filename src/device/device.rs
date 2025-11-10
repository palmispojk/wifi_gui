use crate::backend::error::NMError;
use crate::device::types::DeviceType;
use std::result::Result;
use zbus::{self, Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Device",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device",
    assume_defaults = true
)]
pub trait Device {
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

pub struct DeviceClient<'a> {
    conn: &'a Connection,
    proxy: DeviceProxy<'a>,
    path: &'a OwnedObjectPath,
}

impl<'a> DeviceClient<'a> {
    pub async fn new(conn: &'a Connection, path: &'a OwnedObjectPath) -> Result<Self, NMError> {
        let proxy = DeviceProxy::builder(conn)
            .path(path)?
            .build()
            .await
            .map_err(|err| NMError::Dbus(err))?;

        Ok(Self { conn, proxy, path })
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }

    pub async fn get_device_type(&self) -> Result<DeviceType, NMError> {
        let raw_type = self
            .proxy
            .device_type()
            .await
            .map_err(|err| NMError::Dbus(err))?;
        Ok(DeviceType::from(raw_type))
    }
}
