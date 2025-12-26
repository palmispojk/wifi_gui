use crate::device::wifi::access_point::NetworkDisplayInfo;
use ratatui::style::Color;

pub trait NetworkDisplayExt {
    fn signal_icon(&self) -> &'static str;
    fn signal_color(&self) -> Color;
}

impl NetworkDisplayExt for NetworkDisplayInfo {
    /// Returns a visual signal strength icon
    fn signal_icon(&self) -> &'static str {
        match self.strength {
            0..=20 => "  ",
            21..=40 => "▂ ",
            41..=60 => "▂▄",
            61..=80 => "▂▄▆",
            _ => "▂▄▆█",
        }
    }

    /// Returns a color based on signal strength
    fn signal_color(&self) -> Color {
        match self.strength {
            0..=30 => Color::Red,
            31..=70 => Color::Yellow,
            _ => Color::Green,
        }
    }
}
