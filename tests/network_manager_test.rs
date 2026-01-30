use std::sync::Arc;
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
