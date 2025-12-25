use std::sync::Arc;
use wifi_gui::backend::error::NMError;
use wifi_gui::backend::network_manager::NetworkManagerClient;

#[tokio::test]
async fn test_create_nm_client() {
    let conn = zbus::Connection::system()
        .await
        .expect("System bus expected!");

    let conn_arc = Arc::new(conn);

    let client = NetworkManagerClient::new(conn_arc.clone()).await;

    assert!(client.is_ok(), "Failed to create NetworkManagerClient");
}

#[tokio::test]
async fn test_get_device_paths() {
    let conn = zbus::Connection::system()
        .await
        .expect("System bus expected!");

    let conn_arc = Arc::new(conn);

    let client = NetworkManagerClient::new(conn_arc.clone()).await.unwrap();

    match client.get_device_paths().await {
        Ok(devices) => {
            assert!(
                !devices.is_empty(),
                "Expected at least one device, got none"
            );
            println!("Devices:");
            for d in devices {
                println!("  {}", d);
            }
        }
        Err(NMError::NoDeviceFound) => {
            panic!("No devices found, but system probably has Wi-Fi or Ethernet")
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
    }
}
