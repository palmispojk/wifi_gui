use ratatui::style::Color;

pub struct NetworkDisplayInfo {
    pub ssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub is_secure: bool,
    pub path: String, // Keep the path for connecting later
}

impl NetworkDisplayInfo {
    /// Returns a visual signal strength icon
    pub fn signal_icon(&self) -> &'static str {
        match self.strength {
            0..=20 => "  ",
            21..=40 => "▂ ",
            41..=60 => "▂▄",
            61..=80 => "▂▄▆",
            _ => "▂▄▆█",
        }
    }

    /// Returns a color based on signal strength
    pub fn signal_color(&self) -> Color {
        match self.strength {
            0..=30 => Color::Red,
            31..=70 => Color::Yellow,
            _ => Color::Green,
        }
    }

    /// Identifies the band (2.4GHz vs 5GHz)
    pub fn band(&self) -> &'static str {
        if self.frequency > 4000 {
            "5GHz"
        } else {
            "2.4GHz"
        }
    }
}
