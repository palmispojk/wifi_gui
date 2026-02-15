use crossterm::event::{Event, EventStream, KeyCode};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};
use tokio::sync::mpsc;

use crate::backend::network_manager::NetworkManagerClient;
use crate::device::wifi::access_point::AccessPointUpdate;
use crate::tui::app::{AppAction, AppState, InputMode};
use crate::tui::handler::handle_connect_input;
use crate::tui::widgets::draw_ui;
pub mod app;
pub mod error;
pub mod handler;
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
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<AppAction>();

        let mut app = AppState::new();
        let mut reader = EventStream::new();

        let nm = Arc::new(
            NetworkManagerClient::new(conn.clone())
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?,
        );

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

                Some(action) = action_rx.recv() => {
                    app.handle_action(action);
                }

                Some(Ok(event)) = reader.next() => {
                    if let Event::Key(key) = event {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            match app.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('q') => break,
                                    KeyCode::Down => app.next(),
                                    KeyCode::Up => app.previous(),
                                    KeyCode::Enter => {
                                        handle_connect_input(&mut app, nm.clone(), action_tx.clone()).await;
                                    }
                                    _ => {}
                                },
                                InputMode::PasswordInput => match key.code {
                                    KeyCode::Enter => {
                                        handle_connect_input(&mut app, nm.clone(), action_tx.clone()).await;
                                    }
                                    KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                        app.password_buffer = None;
                                    }
                                    KeyCode::Char(c) => {
                                        let buf = app.password_buffer.get_or_insert_with(String::new);
                                        buf.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        if let Some(ref mut buf) = app.password_buffer {
                                            buf.pop();
                                        }
                                    }
                                    _ => {}
                                }
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
