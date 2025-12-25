use std::sync::Arc;

use crate::device::device::DeviceClient;
use crate::{backend::error::NMError, device::wifi::access_point::AccessPointClient};

use zbus::Connection;
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
trait Wireless {
    fn get_all_access_points(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn request_scan(&self, options: std::collections::HashMap<&str, Value<'_>>)
    -> zbus::Result<()>;

    #[zbus(property)]
    fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    #[zbus(property)]
    fn access_points(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

pub struct WirelessClient {
    proxy: WirelessProxy<'static>,
    path: OwnedObjectPath,
    conn: Arc<Connection>,
}

impl WirelessClient {
    pub async fn new(device: &DeviceClient) -> Result<Self, NMError> {
        let path = device.path().clone();

        let conn = device.conn().clone();

        let proxy = WirelessProxy::builder(&conn)
            .path(path.clone())?
            .build()
            .await?;

        Ok(Self { proxy, path, conn })
    }

    pub async fn list_access_points(&self) -> Result<Vec<AccessPointClient>, NMError> {
        let ap_paths = self.proxy.get_all_access_points().await?;
        let mut access_points = Vec::with_capacity(ap_paths.len());
        for path in ap_paths.into_iter() {
            let ap_client = AccessPointClient::new(self.conn.clone(), path).await?;
            access_points.push(ap_client);
        }

        Ok(access_points)
    }

    pub async fn scan(&self) -> Result<(), NMError> {
        self.proxy
            .request_scan(std::collections::HashMap::new())
            .await
            .map_err(|err| NMError::Dbus(err))
    }

    pub async fn get_active_ap(&self) -> Result<OwnedObjectPath, NMError> {
        let active_ap = self
            .proxy
            .active_access_point()
            .await
            .map_err(|_| NMError::NoDeviceFound)?;
        Ok(active_ap)
    }
}
