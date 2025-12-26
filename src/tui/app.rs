use crate::device::wifi::access_point::{AccessPointUpdate, NetworkDisplayInfo};
use ratatui::widgets::ListState;

pub struct AppState {
    pub networks: Vec<NetworkDisplayInfo>,
    pub list_state: ListState,
    pub is_scanning: bool,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn apply_update(&mut self, update: AccessPointUpdate) {
        match update {
            AccessPointUpdate::PropertyChanged { path, strength } => {
                if let Some(network) = self.networks.iter_mut().find(|n| n.path == path) {
                    network.strength = strength;
                }
            }

            AccessPointUpdate::Added(new_network) => {
                self.networks.push(new_network);
            }

            AccessPointUpdate::Removed(path) => {
                self.networks.retain(|n| n.path != path);
            }
        }
    }

    pub fn new() -> Self {
        Self {
            networks: Vec::new(),
            list_state: ListState::default(),
            is_scanning: true,
            error_message: None,
        }
    }

    pub fn set_networks(&mut self, networks: Vec<NetworkDisplayInfo>) {
        self.networks = networks;
        self.is_scanning = false;

        if self.list_state.selected().is_none() && !self.networks.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.networks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.networks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.networks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.networks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));
    }
}
