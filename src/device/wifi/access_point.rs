use std::sync::Arc;

use crate::device::wifi::sec_types::{NM80211ApFlags, NM80211ApSecFlags, WifiSecurity};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

use crate::backend::error::NMError;

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

pub struct AccessPointClient {
    proxy: AccessPointProxy<'static>,
    path: OwnedObjectPath,
}

impl AccessPointClient {
    pub async fn new(conn: Arc<Connection>, path: OwnedObjectPath) -> Result<Self, NMError> {
        let proxy = AccessPointProxy::builder(&conn)
            .path(path.clone())?
            .build()
            .await?;

        Ok(Self { proxy, path })
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }

    pub async fn ssid(&self) -> Result<String, NMError> {
        let raw_ssid = self.proxy.ssid().await?;
        Ok(String::from_utf8_lossy(&raw_ssid).into_owned())
    }

    pub async fn strength(&self) -> Result<u8, NMError> {
        let strength = self.proxy.strength().await?;
        Ok(strength)
    }

    pub async fn frequency(&self) -> Result<u32, NMError> {
        let frequency = self.proxy.frequency().await?;
        Ok(frequency)
    }

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
