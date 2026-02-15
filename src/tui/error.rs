use std;
use std::fmt;

#[derive(Debug)]
pub enum FrontendError {
    /// Already trying to connect to a network
    AlreadyConnecting,
    /// Tui error with the given external tui crate
    Tui(String),
    /// System frontend error not related to tui crate
    System(String),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrontendError::AlreadyConnecting => {
                write!(f, "Already working on connecting to a network")
            }
            FrontendError::Tui(msg) => {
                write!(f, "An error occured in the Tui with message: {}", msg)
            }
            FrontendError::System(msg) => {
                write!(f, "A system error occured with message: {}", msg)
            }
        }
    }
}

impl std::error::Error for FrontendError {}
