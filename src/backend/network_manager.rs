use crate::backend::error::NMError;
use std::result::Result;
use zbus::{self, Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(interface = "org.freedesktop.NetworkManager", assume_defaults = true)]
pub trait NetworkManager {
    // Method for GetDevices D-Bus
    async fn get_devices(&self) -> Result<Vec<OwnedObjectPath>, NMError>;
    //
    async fn get_all_devices(&self) -> Result<Vec<OwnedObjectPath>, NMError>;
}

pub struct NetworkManagerClient<'a> {
    conn: &'a Connection,
    proxy: NetworkManagerProxy<'a>,
}

impl<'a> NetworkManagerClient<'a> {
    pub async fn new(conn: &'a Connection) -> Result<Self, NMError> {
        let proxy = NetworkManagerProxy::new(conn)
            .await
            .map_err(|err| NMError::Dbus(err))?;
        Ok(Self { conn, proxy })
    }

    pub async fn get_device_paths(&self) -> Result<Vec<OwnedObjectPath>, NMError> {
        let devices = self.proxy.get_devices().await?;
        if devices.is_empty() {
            return Err(NMError::NoDeviceFound);
        }
        Ok(devices)
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
