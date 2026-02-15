use std::sync::Arc;

use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

use crate::backend::error::NMError;

impl SettingsConnectionProxy<'_> {
    pub async fn new_from_path(
        device_path: OwnedObjectPath,
        conn: Arc<Connection>,
    ) -> Result<Self, NMError> {
        let proxy = SettingsConnectionProxy::builder(&conn)
            .path(device_path)?
            .build()
            .await?;

        Ok(proxy)
    }
}

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Settings/Connection",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    assume_defaults = true
)]
pub trait SettingsConnection {
    fn delete(&self) -> zbus::Result<()>;

    fn get_settings(
        &self,
    ) -> zbus::Result<
        std::collections::HashMap<
            String,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
    >;
}
