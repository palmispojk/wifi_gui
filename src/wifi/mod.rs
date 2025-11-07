use crate::wifi::error::WifiError;
use async_trait::async_trait;

pub mod error;
pub mod nm;

#[async_trait]
pub trait WifiBackend {
    fn scan(&self) -> Result<Vec<WifiNetwork>, WifiError>;
    fn connect(&self, ssid: &str, password: Option<&str>) -> Result<(), WifiError>;
    fn current(&self) -> Result<Option<WifiNetwork>, WifiError>;
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub strength: u8,
    pub secured: bool,
}
