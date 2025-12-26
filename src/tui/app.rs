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

#[cfg(test)]
mod test {
    use super::*;

    fn mock_network(ssid: &str, strength: u8, path: &str) -> NetworkDisplayInfo {
        NetworkDisplayInfo {
            ssid: ssid.into(),
            strength,
            path: path.into(),
            band: "5GHz".into(),
            is_secure: true,
        }
    }

    /// Verifies that multiple access points with the same SSID are deduplicated,
    /// keeping only the one with the strongest signal.
    ///
    /// # Scenario
    /// 1. Add a network with SSID "A" (50% strength).
    /// 2. Add another network with SSID "A" (80% strength).
    /// 3. Assert the state only contains the 80% strength version.
    #[test]
    fn test_deduplication_on_added() {
        let mut app = AppState::new();
        let net1 = mock_network("A", 50, "/1");
        let net2 = mock_network("A", 80, "/2");

        app.apply_update(AccessPointUpdate::Added(net1));
        app.apply_update(AccessPointUpdate::Added(net2));

        assert_eq!(app.networks.len(), 1);
        assert_eq!(app.networks[0].strength, 80);
        assert_eq!(app.networks[0].path, "/2");
    }

    /// Verifies that the UI selection "sticks" to the chosen hardware (path)
    /// even if its position in the list changes due to sorting.
    ///
    /// # Scenario
    /// 1. Start with two networks where "B" is at the bottom.
    /// 2. User selects "B" (index 1).
    /// 3. "B" signal jumps to 90%, causing it to move to index 0.
    /// 4. Assert that `list_state` updates to index 0, keeping "B" highlighted.
    #[test]
    fn test_selection_persistence_after_sort() {
        let mut app = AppState::new();
        let net1 = mock_network("A", 50, "/1");
        let net2 = mock_network("B", 30, "/2");

        app.set_networks(vec![net1.clone(), net2.clone()]);
        app.selected_path = Some("/2".into());

        app.apply_update(AccessPointUpdate::PropertyChanged {
            path: "/2".into(),
            strength: 90,
        });

        assert_eq!(app.list_state.selected(), Some(0));
        assert_eq!(app.selected_path.as_ref().unwrap(), "/2");
    }

    /// Ensures that if the currently selected network is removed from the list,
    /// the application gracefully falls back to selecting the next available item.
    ///
    /// This prevents the UI from panicking or showing a ghost selection.
    ///
    /// # Scenario
    /// 1. Select the second item in a two-item list.
    /// 2. Remove that second item via a backend signal.
    /// 3. Assert the selection resets to the first remaining item.
    #[test]
    fn test_remove_selected_item() {
        // test if logic is correct when removing the selected item
        let mut app = AppState::new();
        let net1 = mock_network("A", 50, "/1");
        let net2 = mock_network("B", 30, "/2");
        app.set_networks(vec![net1.clone(), net2.clone()]);

        app.next();
        app.apply_update(AccessPointUpdate::Removed("/2".to_string()));

        assert_eq!(app.networks.len(), 1);
        assert_eq!(app.list_state.selected(), Some(0));
        assert_eq!(app.selected_path, Some("/1".to_string()));
    }

    /// Tests that the application state resets correctly when the only available network is removed.
    ///
    /// This ensures that the UI doesn't attempt to maintain a selection on a non-existent
    /// path, which prevents index-out-of-bounds errors and "ghost" highlights during rendering.
    ///
    /// # Scenario
    /// 1. Initialize the app with a single network.
    /// 2. Receive a signal that this specific network has been removed.
    /// 3. Assert that the data list, the unique path tracker, and the visual
    ///    selection state are all cleared.
    #[test]
    fn test_remove_last_item() {
        let mut app = AppState::new();
        let net1 = mock_network("A", 50, "/1");

        app.set_networks(vec![net1.clone()]);

        app.apply_update(AccessPointUpdate::Removed("/1".to_string()));

        assert_eq!(app.networks.len(), 0);
        assert_eq!(app.selected_path, None);
        assert_eq!(app.list_state.selected(), None);
    }
}
