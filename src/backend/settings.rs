use zbus::{proxy, zvariant::OwnedObjectPath};

use crate::backend::error::NMError;

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Settings",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Settings",
    assume_defaults = true
)]
pub trait Settings {
    fn list_connections(&self) -> Result<Vec<OwnedObjectPath>, NMError>;
}
