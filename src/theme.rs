use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub highlight_bg: String,
    pub highlight_fg: String,
    pub header_1: String,
    pub header_2: String,
    pub header_3: String,
    pub code_bg: String,
    pub encrypted_tag: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary: "#FF8C00".to_string(),      // Warm Dark Orange
            secondary: "#F39C12".to_string(),    // Amber / Warm Yellow
            accent: "#FFD700".to_string(),       // Gold / Yellow
            background: "none".to_string(),      // Transparent background by default
            foreground: "#FFF8DC".to_string(),    // Cornsilk light text
            border: "#E67E22".to_string(),        // Warm border
            highlight_bg: "#3D2400".to_string(),  // Warm dark brown/amber highlight
            highlight_fg: "#FFD700".to_string(),  // Gold highlight text
            header_1: "#FF5500".to_string(),      // Deep Warm Orange Header 1
            header_2: "#FFA500".to_string(),      // Bright Orange Header 2
            header_3: "#FFD700".to_string(),      // Gold Header 3
            code_bg: "#24180E".to_string(),       // Dark warm brown code block bg
            encrypted_tag: "#E74C3C".to_string(), // Crimson red for encrypted notes
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub header_1: Color,
    pub header_2: Color,
    pub header_3: Color,
    pub code_bg: Color,
    pub encrypted_tag: Color,
    pub transparent: bool,
}

impl Theme {
    pub fn from_config(config: &ThemeConfig, transparent: bool) -> Self {
        Self {
            primary: parse_color(&config.primary),
            secondary: parse_color(&config.secondary),
            accent: parse_color(&config.accent),
            background: if transparent {
                Color::Reset
            } else {
                parse_color(&config.background)
            },
            foreground: parse_color(&config.foreground),
            border: parse_color(&config.border),
            highlight_bg: parse_color(&config.highlight_bg),
            highlight_fg: parse_color(&config.highlight_fg),
            header_1: parse_color(&config.header_1),
            header_2: parse_color(&config.header_2),
            header_3: parse_color(&config.header_3),
            code_bg: parse_color(&config.code_bg),
            encrypted_tag: parse_color(&config.encrypted_tag),
            transparent,
        }
    }

    pub fn bg_style(&self) -> Style {
        if self.transparent {
            Style::default().bg(Color::Reset)
        } else {
            Style::default().bg(self.background)
        }
    }

    pub fn fg_style(&self) -> Style {
        self.bg_style().fg(self.foreground)
    }

    pub fn border_style(&self) -> Style {
        // Inactive border style: subtle dim charcoal/warm gray
        self.bg_style().fg(Color::DarkGray)
    }

    pub fn active_border_style(&self) -> Style {
        // Active border style: vibrant glowing gold with bold modifier
        self.bg_style().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn title_style(&self) -> Style {
        self.bg_style().fg(Color::DarkGray)
    }

    pub fn active_title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .bg(self.highlight_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(self.highlight_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_active_style(&self) -> Style {
        Style::default()
            .bg(self.primary)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_inactive_style(&self) -> Style {
        self.bg_style().fg(Color::DarkGray)
    }
}

pub fn parse_color(color_str: &str) -> Color {
    let s = color_str.trim().to_lowercase();
    if s == "none" || s == "transparent" || s == "reset" {
        return Color::Reset;
    }

    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        } else if hex.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..1].repeat(2), 16),
                u8::from_str_radix(&hex[1..2].repeat(2), 16),
                u8::from_str_radix(&hex[2..3].repeat(2), 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }

    match s.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        "orange" => Color::Rgb(255, 140, 0),
        "amber" => Color::Rgb(255, 191, 0),
        "gold" => Color::Rgb(255, 215, 0),
        _ => Color::Reset,
    }
}
