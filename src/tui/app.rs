use ratatui::widgets::ListState;

use crate::device::wifi::access_point::NetworkDisplayInfo;

pub struct AppState {
    pub networks: Vec<NetworkDisplayInfo>,
    pub list_state: ListState,
    pub is_scanning: bool,
    pub error_message: Option<String>,
}

impl AppState {
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
