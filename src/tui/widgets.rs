use crate::tui::app::AppState;
use crate::tui::models::NetworkDisplayExt;
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
}

fn render_network_list(f: &mut Frame, app: &mut AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .networks
        .iter()
        .map(|net| {
            let security_icon = if net.is_secure { "" } else { "" };

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
                Span::raw(security_icon),
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
    let status = if app.is_scanning {
        "Scanning..."
    } else {
        "Idle"
    };
    let text = format!(" [q] Quit | [↑↓] Navigate | Status: {}", status);

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}
