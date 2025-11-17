use crate::{backend::error::NMError, device::wifi::sec_types::WifiSecurity};
use std::result::Result;
use zbus::{
    self, Connection, proxy,
    zvariant::{OwnedObjectPath, Value},
};

/// D-Bus proxy interface for `org.freedesktop.NetworkManager`.
///
/// Provides async methods to retrieve device object paths from NetworkManager.
#[proxy(interface = "org.freedesktop.NetworkManager", assume_defaults = true)]
pub trait NetworkManager {
    // Method for GetDevices D-Bus
    async fn get_devices(&self) -> Result<Vec<OwnedObjectPath>, NMError>;
    //
    async fn get_all_devices(&self) -> Result<Vec<OwnedObjectPath>, NMError>;

    async fn add_and_activate_connection(
        &self,
        connection: Value<'_>,
        device: &OwnedObjectPath,
        specific_object: &OwnedObjectPath,
    ) -> Result<(OwnedObjectPath, OwnedObjectPath), NMError>;
}

/// Client for interacting with the NetworkManager service.
///
/// Holds a D-Bus connection and a proxy for calling NetworkManager methods.
pub struct NetworkManagerClient<'a> {
    conn: &'a Connection,
    proxy: NetworkManagerProxy<'a>,
}

impl<'a> NetworkManagerClient<'a> {
    /// Creates a new `NetworkManagerClient` using the given D-Bus connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to a live D-Bus `Connection`.
    ///
    /// # Returns
    ///
    /// Returns a `NetworkManagerClient` or an `NMError` if
    /// the proxy cannot be built.
    pub async fn new(conn: &'a Connection) -> Result<Self, NMError> {
        let proxy = NetworkManagerProxy::new(conn).await?;
        Ok(Self { conn, proxy })
    }

    /// Returns the paths of all active devices.
    ///
    /// # Errors
    ///
    /// Returns `NMError::NoDeviceFound` if no devices are available.
    pub async fn get_device_paths(&self) -> Result<Vec<OwnedObjectPath>, NMError> {
        let devices = self.proxy.get_devices().await?;
        if devices.is_empty() {
            return Err(NMError::NoDeviceFound);
        }
        Ok(devices)
    }

    pub async fn connect_to_wifi(
        &self,
        device_path: &OwnedObjectPath,
        ap_path: &OwnedObjectPath,
        security: WifiSecurity,
        password: Option<&str>,
    ) -> Result<(), NMError> {
        let connection_dict = Value::from(security.to_nm_dict(password));

        self.proxy
            .add_and_activate_connection(connection_dict, device_path, ap_path)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::OwnedObjectPath;

    #[tokio::test]
    async fn test_no_device_error_logic() {
        let devices: Vec<OwnedObjectPath> = vec![];

        let result = if devices.is_empty() {
            Err(NMError::NoDeviceFound)
        } else {
            Ok(devices)
        };

        assert!(matches!(result, Err(NMError::NoDeviceFound)));
    }
}
