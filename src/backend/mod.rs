pub mod error;
pub mod network_manager;

use crate::backend::error::NMError;
use async_trait::async_trait;

#[async_trait]
pub trait WifiBackend {
    fn scan(&self) -> Result<Vec<WifiNetwork>, NMError>;
    fn connect(&self, ssid: &str, password: Option<&str>) -> Result<(), NMError>;
    fn current(&self) -> Result<Option<WifiNetwork>, NMError>;
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub strength: u8,
    pub secured: bool,
}
