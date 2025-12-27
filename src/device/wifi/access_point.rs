use std::sync::Arc;

use crate::device::wifi::sec_types::{NM80211ApFlags, NM80211ApSecFlags, WifiSecurity};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

use crate::backend::error::NMError;

/// Proxy for Access Points with the (for now)
#[proxy(
    default_path = "/org/freedesktop/NetworkManager/AccessPoint",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    assume_defaults = true
)]
trait AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;

    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;

    ///frequency property in MHz
    #[zbus(property)]
    fn frequency(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn flags(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn wpa_flags(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn rsn_flags(&self) -> zbus::Result<u32>;
}

/// Access point struct for handling the proxy and path
pub struct AccessPointClient {
    proxy: AccessPointProxy<'static>,
    path: OwnedObjectPath,
}

impl AccessPointClient {
    /// New function that creates and stores the proxy and path
    ///
    /// # Arguments
    /// `conn` - The dbus Connection
    /// `path` - The dbus path to the access point
    pub async fn new(conn: Arc<Connection>, path: OwnedObjectPath) -> Result<Self, NMError> {
        let proxy = AccessPointProxy::builder(&conn)
            .path(path.clone())?
            .build()
            .await?;

        Ok(Self { proxy, path })
    }

    /// Function that returns a reference to the access point path
    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }

    /// Returns the ssid of the access point from the proxy
    ///
    /// # Errors
    ///
    /// `NMError` - from dbus error
    pub async fn ssid(&self) -> Result<String, NMError> {
        let raw_ssid = self.proxy.ssid().await?;
        Ok(String::from_utf8_lossy(&raw_ssid).into_owned())
    }

    /// Returns the strength of the access point connection from the proxy
    ///
    /// # Errors
    ///
    /// `NMError` - from dbus error
    pub async fn strength(&self) -> Result<u8, NMError> {
        let strength = self.proxy.strength().await?;
        Ok(strength)
    }

    /// Returns the frequency of the access point from the proxy
    ///
    /// # Errors
    ///
    /// `NMError` - from dbus error
    pub async fn frequency(&self) -> Result<u32, NMError> {
        let frequency = self.proxy.frequency().await?;
        Ok(frequency)
    }

    /// Returns the sequrity flag to `WifiSecurity` from the proxy
    ///
    /// # Errors
    ///
    /// `NMError` - from dbus error
    pub async fn security(&self) -> Result<WifiSecurity, NMError> {
        let wpa = self.proxy.wpa_flags().await?;
        let rsn = self.proxy.rsn_flags().await?;

        let flags = NM80211ApSecFlags::from_bits_truncate(wpa | rsn);
        Ok(WifiSecurity::from(flags))
    }

    pub async fn flags(&self) -> Result<NM80211ApFlags, NMError> {
        let raw = self.proxy.flags().await?;
        Ok(NM80211ApFlags::from_bits_truncate(raw))
    }
}

/// Enum to handle the transmitting of dynamic updates from listening to the access points
pub enum AccessPointUpdate {
    Added(NetworkDisplayInfo),
    Removed(String),
    PropertyChanged { path: String, strength: u8 },
}

/// Struct to handle the displaying of an access point
#[derive(Debug, Clone)]
pub struct NetworkDisplayInfo {
    pub ssid: String,
    pub strength: u8,
    pub band: &'static str,
    pub is_secure: bool,
    pub path: String,
}

impl NetworkDisplayInfo {
    /// Creates a new display for an access point
    ///
    /// # Errors
    ///
    /// `NMError` - from dbus error
    pub async fn new(ap: &AccessPointClient) -> Result<Self, NMError> {
        Ok(Self {
            ssid: ap.ssid().await.unwrap_or_else(|_| "<hidden>".into()),
            strength: ap.strength().await.unwrap_or(0),
            band: if ap.frequency().await.unwrap_or(0) > 4000 {
                "5GHz"
            } else {
                "2.4GHz"
            },
            is_secure: ap.security().await.is_ok(),
            path: ap.path().to_string(),
        })
    }
}
