use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::mpsc;

use crate::backend::network_manager::NetworkManagerClient;
use crate::tui::app::AppState;
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
                let mut interval = tokio::time::interval(Duration::from_secs(30));
                loop {
                    if let Ok(display_list) = nm.scan_all_wifi_networks().await {
                        let _ = tx.send(display_list).await;
                    }
                    interval.tick().await;
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
