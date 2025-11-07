use zbus::{self, Connection};

use crate::wifi::error::WifiError;

pub struct NetworkManager {
    conn: Connection,
    device_path: String,
}

impl NetworkManager {
    pub async fn new() -> Result<Self, WifiError> {
        let conn = Connection::system().await?;

        let proxy = zbus::Proxy::new(
            &conn,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
        )
        .await?;

        let devices: Vec<zbus::zvariant::OwnedObjectPath> =
            proxy.call("GetAllDevices", &()).await?;

        let wifi_device_path = devices
            .into_iter()
            .find(|device| {
                // just take the first now for testing.
                true
            })
            .ok_or(WifiError::NoDeviceFound)?;

        Ok(NetworkManager {
            conn,
            device_path: wifi_device_path.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_new_network_manager() {
        let nm_result = NetworkManager::new().await;

        match nm_result {
            Ok(nm) => {
                assert!(!nm.device_path.is_empty(), "Device path was empty!");
            }
            Err(err) => match err {
                WifiError::NoDeviceFound => {
                    panic!(
                        "Possible sucess if no device is installed on the device: {}",
                        err
                    );
                }
                _ => {
                    panic!("Unexpected error ocurred: {}", err);
                }
            },
        }
    }
}
