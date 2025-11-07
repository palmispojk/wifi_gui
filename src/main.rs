mod wifi;
use crate::wifi::nm::NetworkManager;

#[tokio::main]
async fn main() {
    match NetworkManager::new().await {
        Ok(nm) => println!("Found Wi-Fi device at path"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
