use crossterm::event::{Event, EventStream, KeyCode};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};
use tokio::sync::mpsc;

use crate::backend::network_manager::NetworkManagerClient;
use crate::device::wifi::access_point::AccessPointUpdate;
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
        let (update_tx, mut update_rx) = mpsc::channel::<AccessPointUpdate>(100);
        let mut app = AppState::new();
        let mut reader = EventStream::new();

        let nm = NetworkManagerClient::new(conn.clone())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if let Ok(initial_networks) = nm.scan_all_wifi_networks().await {
            app.set_networks(initial_networks);
        }

        nm.listen_for_changes(update_tx)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        loop {
            terminal.draw(|f| draw_ui(f, &mut app))?;
            tokio::select! {
                Some(update) = update_rx.recv() => {
                    app.apply_update(update);
                }

                Some(Ok(event)) = reader.next() => {
                    if let Event::Key(key) = event {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => break,
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
