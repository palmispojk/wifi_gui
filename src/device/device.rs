use crate::backend::error::NMError;
use crate::device::types::DeviceType;
use std::{result::Result, sync::Arc};
use zbus::{self, Connection, proxy, zvariant::OwnedObjectPath};

/// D-Bus proxy interface for a NetworkManager device
#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Device",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device",
    assume_defaults = true
)]
pub trait Device {
    /// The D-Bus `DeviceType` property.
    ///
    /// Returns the raw numeric type of the device (e.g., 2 for Wi-Fi).
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

///Client for interacting with a single NetworkManager device.
///
///Holds a D-Bus proxy and a reference to the device path. Provides
///methods to query device properties asynchronously.
pub struct DeviceClient {
    conn: Arc<Connection>,
    proxy: DeviceProxy<'static>,
    path: OwnedObjectPath,
}

impl DeviceClient {
    /// Creates a new `DeviceClient`.
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to an active D-Bus `Connection`.
    /// * `path` - A reference to the device's D-Bus object path.
    ///
    /// # Returns
    ///
    ///`Result<Self, NMError>` - Returns the client or
    /// an error if the proxy cannot be built.
    pub async fn new(conn: Arc<Connection>, path: OwnedObjectPath) -> Result<Self, NMError> {
        let proxy = DeviceProxy::builder(&conn)
            .path(path.clone())?
            .build()
            .await?;

        Ok(Self { conn, proxy, path })
    }

    /// Returns the object path of the device.
    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }

    pub fn conn(&self) -> &Arc<Connection> {
        &self.conn
    }

    /// Returns the device type as a `DeviceType` enum.
    ///
    ///Internally queries the `DeviceType` D-Bus property asynchronously.
    pub async fn get_device_type(&self) -> Result<DeviceType, NMError> {
        let raw_type = self.proxy.device_type().await?;
        Ok(DeviceType::from(raw_type))
    }
}
