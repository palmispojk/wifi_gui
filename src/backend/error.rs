use std::fmt;
use zbus::Error as ZbusError;

///Error type for all NetworkManager-related operations.
///
///This wraps lower-level D-Bus errors and provides
///semantic errors relevant to Wi-Fi and device management.
#[derive(Debug)]
pub enum NMError {
    /// A D-Bus-level error occurred.
    Dbus(ZbusError),
    /// No NetworkManager-related device was found on the system.
    NoDeviceFound,
    /// The provided SSID was empty or invalid.
    InvalidSsid(String),
    /// Connection attempt to a given SSID failed.
    ConnectionFailed(String),
    /// Stream fail with D-Bus
    SignalError(String),
    /// Network not found error
    NetworkNotFound(String),
}

/// Display implementation for NMError
impl fmt::Display for NMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NMError::Dbus(err) => write!(f, "D-bus error: {}", err),
            NMError::NoDeviceFound => write!(f, "No device found!"),
            NMError::InvalidSsid(ssid) => write!(f, "Invalid SSID {}", ssid),
            NMError::ConnectionFailed(ssid) => write!(f, "Failed to connect to SSID {}", ssid),
            NMError::SignalError(msg) => write!(f, "Signal error: {}", msg),
            NMError::NetworkNotFound(msg) => write!(f, "Network not found: {}", msg),
        }
    }
}

impl std::error::Error for NMError {}

/// Allows automatic conversion from `zbus::Error` to this error type.
///
/// This enables usage of `?` in async proxy/call code returning `NMError`.
impl From<ZbusError> for NMError {
    fn from(err: ZbusError) -> Self {
        NMError::Dbus(err)
    }
}
