use wifi_gui::backend::network_manager::NetworkManagerClient;
use wifi_gui::device::device::DeviceClient;
use zbus::{self, Connection};

#[tokio::main]
async fn main() {
    let conn = Connection::system().await.unwrap();

    let nm = match NetworkManagerClient::new(&conn).await {
        Ok(client) => {
            print!("Connected to NetworkManager");
            client
        }
        Err(err) => {
            eprintln!("Failed to create NetworkManagerClient: {}", err);
            return;
        }
    };

    let devices = match nm.get_device_paths().await {
        Ok(list) => list,
        Err(err) => {
            eprintln!("Failed to get devices: {}", err);
            return;
        }
    };

    if let Some(first_dev) = devices.first() {
        println!("\nInspecting first device: {}", first_dev);

        let device_client = DeviceClient::new(&conn, first_dev).await.unwrap();
        let device_type = device_client.get_device_type().await.unwrap();

        println!("Device type: {:?}", device_type)
    } else {
        println!("⚠️ No devices found.");
    }
}
