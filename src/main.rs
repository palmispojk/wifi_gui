use wifi_gui::backend::{error::NMError, network_manager::NetworkManagerClient};
use wifi_gui::device::device::DeviceClient;
use wifi_gui::device::types::DeviceType;
use wifi_gui::device::wifi::wifi_device::WirelessClient;
use zbus::{self, Connection};

#[tokio::main]
async fn main() -> Result<(), NMError> {
    let conn = Connection::system().await?;

    let nm = NetworkManagerClient::new(&conn).await?;

    let devices = nm.get_device_paths().await?;

    for device_path in &devices {
        let device_client = DeviceClient::new(&conn, device_path).await?;

        if !matches!(device_client.get_device_type().await?, DeviceType::Wifi) {
            continue;
        }

        let wifi_client = WirelessClient::new(&device_client).await?;
        wifi_client.scan().await?;

        let access_points = wifi_client.list_access_points().await?;
        println!("Found {} access points:", access_points.len());

        for ap in access_points {
            let ssid = ap.ssid().await.unwrap_or_else(|_| "<hidden>".into());
            let strength = ap.strength().await.unwrap_or(0);
            let freq = ap.frequency().await.unwrap_or(0);
            let security = ap.security().await.unwrap();

            println!(
                "SSID: {:<25} | Strength: {:>3}% | Freq: {:>4}MHz | Sec: {:?}",
                ssid, strength, freq, security
            );
        }
    }

    Ok(())
}
