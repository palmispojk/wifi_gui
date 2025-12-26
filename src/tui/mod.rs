use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::mpsc;

use crate::backend::error::NMError;
use crate::backend::network_manager::NetworkManagerClient;
use crate::device::device::DeviceClient;
use crate::device::types::DeviceType;
use crate::device::wifi::wifi_device::WirelessClient;
use crate::tui::app::AppState;
use crate::tui::models::NetworkDisplayInfo;
use crate::tui::widgets::draw_ui;

pub mod app;
pub mod models;
pub mod widgets;

pub struct TuiManager;

impl TuiManager {
    pub async fn run(conn: Arc<zbus::Connection>) -> io::Result<()> {
        // --- 1. Terminal Setup ---
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        // --- 2. App State & Communication ---
        let (tx, mut rx) = mpsc::channel(20);
        let mut app = AppState::new();

        // --- 3. Background Scanner ---
        let scan_conn = conn.clone();
        tokio::spawn(async move {
            if let Ok(nm) = NetworkManagerClient::new(scan_conn.clone()).await {
                // Instead of a tight loop, perform one initial scan
                if let Ok(display_list) = perform_full_scan(&nm, &scan_conn).await {
                    let _ = tx.send(display_list).await;
                }

                // Then, set a long interval or wait for a specific trigger
                let mut interval = tokio::time::interval(Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    if let Ok(display_list) = perform_full_scan(&nm, &scan_conn).await {
                        let _ = tx.send(display_list).await;
                    }
                }
            }
        });

        // --- 4. Main Loop ---
        loop {
            terminal.draw(|f| draw_ui(f, &mut app))?; // Render View

            tokio::select! {
                Some(networks) = rx.recv() => app.set_networks(networks), // Update Model
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    if event::poll(Duration::from_millis(0))? {
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Char('q') => break, // Exit loop
                                KeyCode::Down => app.next(),
                                KeyCode::Up => app.previous(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // --- 5. Cleanup ---
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        Ok(())
    }
}

async fn perform_full_scan(
    nm: &NetworkManagerClient,
    conn: &Arc<zbus::Connection>,
) -> Result<Vec<NetworkDisplayInfo>, NMError> {
    let mut results = Vec::new();
    let paths = nm.get_device_paths().await?;

    for path in paths {
        let dev = DeviceClient::new(conn.clone(), path).await?;

        if matches!(dev.get_device_type().await?, DeviceType::Wifi) {
            let wifi = WirelessClient::new(&dev).await?;

            let _ = wifi.scan().await?;

            let aps = wifi.list_access_points().await?;
            for ap in aps {
                results.push(NetworkDisplayInfo {
                    ssid: ap.ssid().await.unwrap_or_else(|_| "<hidden>".into()),
                    strength: ap.strength().await.unwrap_or(0),
                    frequency: ap.frequency().await.unwrap_or(0),
                    is_secure: ap.security().await.is_ok(),
                    path: ap.path().to_string(),
                });
            }
        }
    }
    Ok(results)
}
