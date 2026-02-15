use crate::tui::app::{AppState, AppStatus, InputMode};
use crate::tui::models::NetworkDisplayExt;
use ratatui::layout::Position;
use ratatui::widgets::Clear;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw_ui(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(f.area());

    render_network_list(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
    if matches!(app.input_mode, InputMode::PasswordInput) {
        render_password_input(f, app);
    }
}

fn render_network_list(f: &mut Frame, app: &mut AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .networks
        .iter()
        .map(|net| {
            let content = Line::from(vec![
                Span::styled(
                    format!(" {:<4}", net.signal_icon()),
                    Style::default().fg(net.signal_color()),
                ),
                Span::raw(format!(" {:<20} ", net.ssid)),
                Span::styled(
                    format!(" {:<5} ", net.band),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(net.security.to_string()),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Found Networks ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Indexed(236))
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_status_bar(f: &mut Frame, app: &AppState, area: Rect) {
    let status = match &app.status {
        Some(AppStatus::Error(err)) => err.as_str(),
        Some(AppStatus::IsConnecting(msg)) => msg.as_str(),
        Some(AppStatus::Status(msg)) => msg.as_str(),
        None => "Idle",
    };
    let text = format!(
        " [q] Quit | [↑↓] Navigate | [Enter] Connect |Status: {}",
        status
    );

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_password_input(f: &mut Frame, app: &AppState) {
    let area = centered_rect(60, 20, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Enter Password (esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let pass_len = app.password_buffer.as_ref().map_or(0, |s| s.len());
    let display_pass = "*".repeat(pass_len);

    let paragraph = Paragraph::new(display_pass).block(block);
    f.render_widget(paragraph, area);

    f.set_cursor_position(Position::new(area.x + pass_len as u16 + 1, area.y + 1));
}
