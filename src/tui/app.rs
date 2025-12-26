use crate::device::wifi::access_point::{AccessPointUpdate, NetworkDisplayInfo};
use ratatui::widgets::ListState;

pub struct AppState {
    pub networks: Vec<NetworkDisplayInfo>,
    pub list_state: ListState,
    pub is_scanning: bool,
    pub error_message: Option<String>,
    pub selected_path: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            networks: Vec::new(),
            list_state: ListState::default(),
            is_scanning: true,
            error_message: None,
            selected_path: None,
        }
    }

    fn sync_selection(&mut self) {
        if let Some(ref path) = self.selected_path {
            if let Some(index) = self.networks.iter().position(|n| &n.path == path) {
                self.list_state.select(Some(index));
                return;
            }
        }

        if !self.networks.is_empty() {
            self.selected_path = Some(self.networks[0].path.clone());
            self.list_state.select(Some(0));
        } else {
            self.selected_path = None;
            self.list_state.select(None);
        }
    }

    pub fn set_networks(&mut self, networks: Vec<NetworkDisplayInfo>) {
        self.networks = networks;
        self.is_scanning = false;
        self.sync_selection();
    }

    pub fn apply_update(&mut self, update: AccessPointUpdate) {
        match update {
            AccessPointUpdate::PropertyChanged { path, strength } => {
                if let Some(network) = self.networks.iter_mut().find(|n| n.path == path) {
                    network.strength = strength;
                }
                self.networks.sort_by(|a, b| b.strength.cmp(&a.strength));
                self.sync_selection();
            }

            AccessPointUpdate::Added(new_network) => {
                if let Some(existing_network) = self
                    .networks
                    .iter_mut()
                    .find(|n| n.ssid == new_network.ssid)
                {
                    if new_network.strength > existing_network.strength {
                        *existing_network = new_network
                    }
                } else {
                    self.networks.push(new_network);
                }
                self.networks.sort_by(|a, b| b.strength.cmp(&a.strength));
                self.sync_selection();
            }

            AccessPointUpdate::Removed(path) => {
                self.networks.retain(|n| n.path != path);
                self.sync_selection();
            }
        }
    }

    pub fn next(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let next_i = (i + 1) % self.networks.len();

        self.selected_path = Some(self.networks[next_i].path.clone());
        self.list_state.select(Some(next_i));
    }

    pub fn previous(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let prev_i = if i == 0 {
            self.networks.len() - 1
        } else {
            i - 1
        };

        self.selected_path = Some(self.networks[prev_i].path.clone());
        self.list_state.select(Some(prev_i));
    }
}
