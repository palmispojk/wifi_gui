use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use zbus::zvariant::OwnedObjectPath;

use crate::{
    backend::{error::NMError, network_manager::NetworkManagerClient},
    device::wifi::sec_types::WifiSecurity,
    tui::{
        app::{AppState, AppStatus, InputMode},
        error::FrontendError,
    },
};

#[derive(Debug)]
pub enum AppAction {
    /// takes the ssid to send what we are connecting to
    ConnectStarted(String),
    /// has the `Result<String, NMError>` for what happened after connecting.
    ConnectFinished(Result<String, NMError>),
    /// To send to the frontend when a FrontendError ocurred.
    FrontendError(FrontendError),
    NMError(NMError),
}

pub async fn handle_connect_input(
    app: &mut AppState,
    nm: Arc<NetworkManagerClient>,
    action_tx: UnboundedSender<AppAction>,
) {
    // if already connected send the action of already connected with the error
    if let Some(AppStatus::IsConnecting(_)) = app.status {
        let _ = action_tx.send(AppAction::FrontendError(FrontendError::AlreadyConnecting));
        return;
    }

    let Ok(network_info) = app.network_from_selected().map_err(|error| {
        let _ = action_tx.send(AppAction::FrontendError(error));
    }) else {
        return;
    };

    if network_info.security != WifiSecurity::Open && app.password_buffer.is_none() {
        app.input_mode = InputMode::PasswordInput;
        return;
    }

    let password = app.password_buffer.clone();

    app.password_buffer = None;
    app.input_mode = InputMode::Normal;

    // send the ssid of the network we are trying to connect to
    let _ = action_tx.send(AppAction::ConnectStarted(network_info.ssid.clone()));

    tokio::spawn(async move {
        let path = match OwnedObjectPath::try_from(network_info.path.clone()) {
            Ok(p) => p,
            Err(e) => {
                let _ = action_tx.send(AppAction::FrontendError(FrontendError::System(format!(
                    "Invalid path with error: {}",
                    e
                ))));
                return;
            }
        };

        let result = nm
            .connect_to_wifi(
                &network_info.ssid,
                &path,
                network_info.security,
                password.as_deref(),
            )
            .await;

        let _ = action_tx.send(AppAction::ConnectFinished(result));
    });
}
