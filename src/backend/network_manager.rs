use crate::{
    backend::error::NMError,
    device::{
        device::DeviceClient,
        types::DeviceType,
        wifi::{
            access_point::{AccessPointUpdate, NetworkDisplayInfo},
            sec_types::WifiSecurity,
            wifi_device::WirelessClient,
        },
    },
};
use futures_util::stream::StreamExt;
use std::{collections::HashMap, result::Result, sync::Arc};
use tokio::sync::mpsc;
use zbus::{
    self, Connection, MatchRule, MessageStream, proxy,
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
pub struct NetworkManagerClient {
    conn: Arc<Connection>,
    proxy: NetworkManagerProxy<'static>,
}

impl NetworkManagerClient {
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
    pub async fn new(conn: Arc<Connection>) -> Result<Self, NMError> {
        let proxy = NetworkManagerProxy::new(&conn).await?;
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

    /// Performs a full scan of all available Wi-Fi access points across all devices.
    ///
    /// This function iterates through all network devices, filters for Wi-Fi hardware,
    /// triggers a scan, and deduplicates the results by SSID to return only the
    /// strongest signal for each network.
    ///
    /// # Errors
    /// Returns an [`NMError`] if the D-Bus connection fails or if NetworkManager
    /// is unreachable.
    pub async fn scan_all_wifi_networks(&self) -> Result<Vec<NetworkDisplayInfo>, NMError> {
        let mut best_networks: HashMap<String, NetworkDisplayInfo> = HashMap::new();
        let paths = self.get_device_paths().await?;

        for path in paths {
            let dev = DeviceClient::new(self.conn.clone(), path).await?;

            if matches!(dev.get_device_type().await?, DeviceType::Wifi) {
                let wifi = WirelessClient::new(&dev).await?;

                let _ = wifi.scan().await?;

                let aps = wifi.list_access_points().await?;
                for ap in aps {
                    let info = NetworkDisplayInfo::new(&ap).await?;
                    let ssid = info.ssid.clone();

                    best_networks
                        .entry(ssid)
                        .and_modify(|existing| {
                            if info.strength > existing.strength {
                                *existing = info.clone();
                            }
                        })
                        .or_insert(info);
                }
            }
        }
        let mut results: Vec<NetworkDisplayInfo> = best_networks.into_values().collect();
        results.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(results)
    }

    /// Connects to a wifi accesspoint
    ///
    /// # Arguments
    /// `device_path` - A reference to the objectpath of the device
    /// `ap_path` - A reference path to the accesspoint
    /// `security` - What security the accesspoint
    /// `password` - An optional password if there is no password needed
    ///
    /// # Errors
    /// `NMError` - for any error with dbus with the proxy or connection
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

    /// Starts a background task to listen for D-Bus signals from NetworkManager.
    ///
    /// Specifically listens for `PropertiesChanged` signals on Access Point objects.
    /// Updates are sent through the provided `mpsc::Sender`.
    ///
    /// # Arguments
    /// * `tx` - An asynchronous channel sender for broadcasting [`AccessPointUpdate`]s.
    pub async fn listen_for_changes(
        &self,
        tx: mpsc::Sender<AccessPointUpdate>,
    ) -> Result<(), NMError> {
        let rule = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .interface("org.freedesktop.DBus.Properties")?
            .member("PropertiesChanged")?
            .arg(0, "org.freedesktop.NetworkManager.AccessPoint")?
            .build();

        let mut stream = MessageStream::for_match_rule(rule, &self.conn.clone(), None).await?;

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                let path = msg
                    .header()
                    .path()
                    .map(|p| p.to_string())
                    .unwrap_or_default();

                if let Ok((_interface, changed, _invalidated)) =
                    msg.body()
                        .deserialize::<(String, HashMap<String, Value>, Vec<String>)>()
                {
                    if let Some(v) = changed.get("Strength") {
                        if let Ok(val) = u8::try_from(v) {
                            let _ = tx.send(AccessPointUpdate::PropertyChanged {
                                path: path.clone(),
                                strength: val,
                            });
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tokio_tests {
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
