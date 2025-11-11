use crate::backend::error::NMError;
use crate::device::device::DeviceClient;
use crate::device::types::DeviceType;

use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, Value},
};

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Wireless",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    assume_defaults = true
)]
pub trait Wireless {
    fn get_all_access_points(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn request_scan(&self, options: std::collections::HashMap<&str, Value<'_>>)
    -> zbus::Result<()>;
}

pub struct WirelessClient<'a> {
    proxy: WirelessProxy<'a>,
    path: &'a OwnedObjectPath,
}

impl<'a> WirelessClient<'a> {
    pub async fn new(device: &'a DeviceClient<'a>) -> Result<Self, NMError> {
        let path = device.path();

        let proxy = WirelessProxy::builder(device.conn())
            .path(path)?
            .build()
            .await?;

        Ok(Self { proxy, path })
    }

    pub async fn list_access_points(&self) -> Result<Vec<OwnedObjectPath>, NMError> {
        let access_points = self.proxy.get_all_access_points().await?;
        Ok(access_points)
    }

    pub async fn scan(&self) -> Result<(), NMError> {
        self.proxy
            .request_scan(std::collections::HashMap::new())
            .await
            .map_err(|err| NMError::Dbus(err))
    }
}
