use std::fmt;
use zbus::Error as ZbusError;

#[derive(Debug)]
pub enum WifiError {
    Dbus(ZbusError),
    NoDeviceFound,
    InvalidSsid(String),
    ConnectionFailed(String),
}

impl fmt::Display for WifiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WifiError::Dbus(err) => write!(f, "D-bus error: {}", err),
            WifiError::NoDeviceFound => write!(f, "No device found!"),
            WifiError::InvalidSsid(ssid) => write!(f, "Invalid SSID {}", ssid),
            WifiError::ConnectionFailed(ssid) => write!(f, "Failed to connect to SSID {}", ssid),
        }
    }
}

impl std::error::Error for WifiError {}

impl From<ZbusError> for WifiError {
    fn from(err: ZbusError) -> Self {
        WifiError::Dbus(err)
    }
}
