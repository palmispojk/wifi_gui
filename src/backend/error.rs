use std::fmt;
use zbus::Error as ZbusError;

#[derive(Debug)]
pub enum NMError {
    Dbus(ZbusError),
    NoDeviceFound,
    InvalidSsid(String),
    ConnectionFailed(String),
}

impl fmt::Display for NMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NMError::Dbus(err) => write!(f, "D-bus error: {}", err),
            NMError::NoDeviceFound => write!(f, "No device found!"),
            NMError::InvalidSsid(ssid) => write!(f, "Invalid SSID {}", ssid),
            NMError::ConnectionFailed(ssid) => write!(f, "Failed to connect to SSID {}", ssid),
        }
    }
}

impl std::error::Error for NMError {}

impl From<ZbusError> for NMError {
    fn from(err: ZbusError) -> Self {
        NMError::Dbus(err)
    }
}
