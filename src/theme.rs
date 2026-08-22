use crate::config::{expand_path, Config};
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Built-in Theme Embedded TOML definitions
pub const THEME_NOTEDOG: &str = include_str!("../themes/notedog.toml");
pub const THEME_NORD: &str = include_str!("../themes/nord.toml");
pub const THEME_CATPPUCCIN_MOCHA: &str = include_str!("../themes/catppuccin-mocha.toml");
pub const THEME_CATPPUCCIN_LATTE: &str = include_str!("../themes/catppuccin-latte.toml");
pub const THEME_CATPPUCCIN_MACCHIATO: &str = include_str!("../themes/catppuccin-macchiato.toml");
pub const THEME_CATPPUCCIN_FRAPPE: &str = include_str!("../themes/catppuccin-frappe.toml");
pub const THEME_DRACULA: &str = include_str!("../themes/dracula.toml");
pub const THEME_GRUVBOX: &str = include_str!("../themes/gruvbox.toml");
pub const THEME_TOKYO_NIGHT: &str = include_str!("../themes/tokyo-night.toml");
pub const THEME_AYU_DARK: &str = include_str!("../themes/ayu-dark.toml");
pub const THEME_SOLARIZED_DARK: &str = include_str!("../themes/solarized-dark.toml");
pub const THEME_MONOKAI: &str = include_str!("../themes/monokai.toml");

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("notedog.toml", THEME_NOTEDOG),
    ("nord.toml", THEME_NORD),
    ("catppuccin-mocha.toml", THEME_CATPPUCCIN_MOCHA),
    ("catppuccin-latte.toml", THEME_CATPPUCCIN_LATTE),
    ("catppuccin-macchiato.toml", THEME_CATPPUCCIN_MACCHIATO),
    ("catppuccin-frappe.toml", THEME_CATPPUCCIN_FRAPPE),
    ("dracula.toml", THEME_DRACULA),
    ("gruvbox.toml", THEME_GRUVBOX),
    ("tokyo-night.toml", THEME_TOKYO_NIGHT),
    ("ayu-dark.toml", THEME_AYU_DARK),
    ("solarized-dark.toml", THEME_SOLARIZED_DARK),
    ("monokai.toml", THEME_MONOKAI),
];

pub fn themes_dir() -> PathBuf {
    Config::config_dir().join("themes")
}

pub fn init_themes_dir(config_dir: &Path) {
    let themes_path = config_dir.join("themes");
    if let Err(e) = fs::create_dir_all(&themes_path) {
        eprintln!("Warning: could not create themes dir {:?}: {}", themes_path, e);
        return;
    }

    for (filename, content) in BUILTIN_THEMES {
        let theme_file = themes_path.join(filename);
        if !theme_file.exists() {
            let _ = fs::write(&theme_file, content);
        }
    }
}

pub fn load_theme_by_name(name: &str) -> ThemeConfig {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return ThemeConfig::default();
    }

    let dir = themes_dir();

    // 1. Try file path in ~/.config/notedog/themes/
    let direct_file = dir.join(trimmed);
    if direct_file.exists() && direct_file.is_file() {
        if let Ok(content) = fs::read_to_string(&direct_file) {
            if let Ok(tc) = toml::from_str::<ThemeConfig>(&content) {
                return tc;
            }
        }
    }

    let toml_file = dir.join(format!("{}.toml", trimmed));
    if toml_file.exists() && toml_file.is_file() {
        if let Ok(content) = fs::read_to_string(&toml_file) {
            if let Ok(tc) = toml::from_str::<ThemeConfig>(&content) {
                return tc;
            }
        }
    }

    // 2. Try arbitrary expanded file path
    let expanded = expand_path(trimmed);
    if expanded.exists() && expanded.is_file() {
        if let Ok(content) = fs::read_to_string(&expanded) {
            if let Ok(tc) = toml::from_str::<ThemeConfig>(&content) {
                return tc;
            }
        }
    }

    // 3. Match against built-in themes by name / aliases
    let normalized = trimmed.to_lowercase();
    let theme_str = match normalized.as_str() {
        "notedog" | "default" | "notedog.toml" => THEME_NOTEDOG,
        "nord" | "nord.toml" => THEME_NORD,
        "catppuccin" | "catputtchin" | "catpuccino" | "catppuccin-mocha" | "catppuccin_mocha" | "mocha" | "catppuccin-mocha.toml" => THEME_CATPPUCCIN_MOCHA,
        "catppuccin-latte" | "catppuccin_latte" | "latte" | "catppuccin-latte.toml" => THEME_CATPPUCCIN_LATTE,
        "catppuccin-macchiato" | "catppuccin_macchiato" | "macchiato" | "catppuccin-macchiato.toml" => THEME_CATPPUCCIN_MACCHIATO,
        "catppuccin-frappe" | "catppuccin_frappe" | "frappe" | "catppuccin-frappe.toml" => THEME_CATPPUCCIN_FRAPPE,
        "dracula" | "dracula.toml" => THEME_DRACULA,
        "gruvbox" | "gruvbox-dark" | "gruvbox_dark" | "gruvbox.toml" => THEME_GRUVBOX,
        "tokyo-night" | "tokyonight" | "tokyo_night" | "tokyo-night.toml" => THEME_TOKYO_NIGHT,
        "ayu" | "ayu-dark" | "ayu_dark" | "ayu-dark.toml" => THEME_AYU_DARK,
        "solarized" | "solarized-dark" | "solarized_dark" | "solarized-dark.toml" => THEME_SOLARIZED_DARK,
        "monokai" | "monokai.toml" => THEME_MONOKAI,
        _ => THEME_NOTEDOG,
    };

    toml::from_str::<ThemeConfig>(theme_str).unwrap_or_default()
}


fn default_active_border() -> String { "#FFCC66".to_string() }
fn default_inactive_border() -> String { "#242936".to_string() }
fn default_sidebar_title() -> String { "#36A3D9".to_string() }
fn default_active_sidebar_border() -> String { "#FF7733".to_string() }
fn default_foreground() -> String { "#B3B1AD".to_string() }
fn default_background() -> String { "none".to_string() }
fn default_highlight_bg() -> String { "#1F2430".to_string() }
fn default_highlight_fg() -> String { "#36A3D9".to_string() }
fn default_header_1() -> String { "#FF7733".to_string() }
fn default_header_2() -> String { "#FFCC66".to_string() }
fn default_header_3() -> String { "#36A3D9".to_string() }
fn default_code_bg() -> String { "#14191F".to_string() }
fn default_encrypted_tag() -> String { "#F07178".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(alias = "primary", default = "default_active_border")]
    pub active_border: String,

    #[serde(alias = "border", default = "default_inactive_border")]
    pub inactive_border: String,

    #[serde(alias = "secondary", default = "default_sidebar_title")]
    pub sidebar_title: String,

    #[serde(alias = "accent", default = "default_active_sidebar_border")]
    pub active_sidebar_border: String,

    #[serde(default = "default_foreground")]
    pub foreground: String,

    #[serde(default = "default_background")]
    pub background: String,

    #[serde(default = "default_highlight_bg")]
    pub highlight_bg: String,

    #[serde(default = "default_highlight_fg")]
    pub highlight_fg: String,

    #[serde(default = "default_header_1")]
    pub header_1: String,
    #[serde(default = "default_header_2")]
    pub header_2: String,
    #[serde(default = "default_header_3")]
    pub header_3: String,
    #[serde(default = "default_code_bg")]
    pub code_bg: String,
    #[serde(default = "default_encrypted_tag")]
    pub encrypted_tag: String,

    // Specific component border & title overrides
    pub notebook_border_active: Option<String>,
    pub notebook_border_inactive: Option<String>,
    pub notebook_title_active: Option<String>,
    pub notebook_title_inactive: Option<String>,

    pub section_border_active: Option<String>,
    pub section_border_inactive: Option<String>,
    pub section_title_active: Option<String>,
    pub section_title_inactive: Option<String>,

    pub note_border_active: Option<String>,
    pub note_border_inactive: Option<String>,
    pub note_title_active: Option<String>,
    pub note_title_inactive: Option<String>,

    pub preview_border_active: Option<String>,
    pub preview_border_inactive: Option<String>,
    pub preview_title_active: Option<String>,
    pub preview_title_inactive: Option<String>,

    // Item colors, backgrounds, and font weights ("bold", "normal", "dim", "italic")
    pub notebook_item_bg: Option<String>,
    pub notebook_item_fg: Option<String>,
    pub notebook_item_weight: Option<String>,
    pub notebook_item_selected_bg: Option<String>,
    pub notebook_item_selected_fg: Option<String>,
    pub notebook_item_selected_weight: Option<String>,
    pub notebook_icon_fg: Option<String>,
    pub notebook_icon_selected_fg: Option<String>,

    pub section_item_bg: Option<String>,
    pub section_item_fg: Option<String>,
    pub section_item_weight: Option<String>,
    pub section_item_selected_bg: Option<String>,
    pub section_item_selected_fg: Option<String>,
    pub section_item_selected_weight: Option<String>,
    pub section_icon_fg: Option<String>,
    pub section_icon_selected_fg: Option<String>,

    pub note_item_bg: Option<String>,
    pub note_item_fg: Option<String>,
    pub note_item_weight: Option<String>,
    pub note_item_selected_bg: Option<String>,
    pub note_item_selected_fg: Option<String>,
    pub note_item_selected_weight: Option<String>,
    pub note_icon_fg: Option<String>,
    pub note_icon_selected_fg: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            active_border: default_active_border(),
            inactive_border: default_inactive_border(),
            sidebar_title: default_sidebar_title(),
            active_sidebar_border: default_active_sidebar_border(),
            foreground: default_foreground(),
            background: default_background(),
            highlight_bg: default_highlight_bg(),
            highlight_fg: default_highlight_fg(),
            header_1: default_header_1(),
            header_2: default_header_2(),
            header_3: default_header_3(),
            code_bg: default_code_bg(),
            encrypted_tag: default_encrypted_tag(),
            notebook_border_active: None,
            notebook_border_inactive: None,
            notebook_title_active: None,
            notebook_title_inactive: None,
            section_border_active: None,
            section_border_inactive: None,
            section_title_active: None,
            section_title_inactive: None,
            note_border_active: None,
            note_border_inactive: None,
            note_title_active: None,
            note_title_inactive: None,
            preview_border_active: None,
            preview_border_inactive: None,
            preview_title_active: None,
            preview_title_inactive: None,
            notebook_item_bg: None,
            notebook_item_fg: None,
            notebook_item_weight: None,
            notebook_item_selected_bg: None,
            notebook_item_selected_fg: None,
            notebook_item_selected_weight: None,
            notebook_icon_fg: None,
            notebook_icon_selected_fg: None,
            section_item_bg: None,
            section_item_fg: None,
            section_item_weight: None,
            section_item_selected_bg: None,
            section_item_selected_fg: None,
            section_item_selected_weight: None,
            section_icon_fg: None,
            section_icon_selected_fg: None,
            note_item_bg: None,
            note_item_fg: None,
            note_item_weight: None,
            note_item_selected_bg: None,
            note_item_selected_fg: None,
            note_item_selected_weight: None,
            note_icon_fg: None,
            note_icon_selected_fg: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub active_border: Color,
    pub inactive_border: Color,
    pub sidebar_title: Color,
    pub active_sidebar_border: Color,
    pub foreground: Color,
    pub background: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub header_1: Color,
    pub header_2: Color,
    pub header_3: Color,
    pub code_bg: Color,
    pub encrypted_tag: Color,
    pub transparent: bool,

    // Legacy fields
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,

    // Component-specific overrides
    pub notebook_border_active: Color,
    pub notebook_border_inactive: Color,
    pub notebook_title_active: Color,
    pub notebook_title_inactive: Color,

    pub section_border_active: Color,
    pub section_border_inactive: Color,
    pub section_title_active: Color,
    pub section_title_inactive: Color,

    pub note_border_active: Color,
    pub note_border_inactive: Color,
    pub note_title_active: Color,
    pub note_title_inactive: Color,

    pub preview_border_active: Color,
    pub preview_border_inactive: Color,
    pub preview_title_active: Color,
    pub preview_title_inactive: Color,

    // Item Background Colors
    pub notebook_item_bg_normal: Color,
    pub notebook_item_bg_selected: Color,
    pub section_item_bg_normal: Color,
    pub section_item_bg_selected: Color,
    pub note_item_bg_normal: Color,
    pub note_item_bg_selected: Color,

    // Item Styles
    pub notebook_item_style_normal: Style,
    pub notebook_item_style_selected: Style,
    pub notebook_icon_style_normal: Style,
    pub notebook_icon_style_selected: Style,

    pub section_item_style_normal: Style,
    pub section_item_style_selected: Style,
    pub section_icon_style_normal: Style,
    pub section_icon_style_selected: Style,

    pub note_item_style_normal: Style,
    pub note_item_style_selected: Style,
    pub note_icon_style_normal: Style,
    pub note_icon_style_selected: Style,
}

impl Theme {
    pub fn load_for_config(config: &Config) -> Self {
        let theme_config = config.load_theme();
        Theme::from_config(&theme_config, config.transparent_background)
    }

    pub fn from_config(config: &ThemeConfig, transparent: bool) -> Self {
        let active_border = parse_color(&config.active_border);
        let inactive_border = parse_color(&config.inactive_border);
        let sidebar_title = parse_color(&config.sidebar_title);
        let active_sidebar_border = parse_color(&config.active_sidebar_border);
        let foreground = parse_color(&config.foreground);
        let background = if transparent { Color::Reset } else { parse_color(&config.background) };
        let highlight_bg = parse_color(&config.highlight_bg);
        let highlight_fg = parse_color(&config.highlight_fg);

        let default_unselected_bg = if transparent { Color::Reset } else { background };

        let notebook_item_bg_normal = config.notebook_item_bg.as_deref().map(parse_color).unwrap_or(default_unselected_bg);
        let notebook_item_bg_selected = config.notebook_item_selected_bg.as_deref().map(parse_color).unwrap_or(highlight_bg);

        let section_item_bg_normal = config.section_item_bg.as_deref().map(parse_color).unwrap_or(default_unselected_bg);
        let section_item_bg_selected = config.section_item_selected_bg.as_deref().map(parse_color).unwrap_or(highlight_bg);

        let note_item_bg_normal = config.note_item_bg.as_deref().map(parse_color).unwrap_or(default_unselected_bg);
        let note_item_bg_selected = config.note_item_selected_bg.as_deref().map(parse_color).unwrap_or(highlight_bg);

        // Helper to construct Style with custom or default fg and font weight
        let make_item_style = |fg_opt: Option<&str>, weight_opt: Option<&str>, default_fg: Color, default_mod: Modifier| -> Style {
            let fg = fg_opt.map(parse_color).unwrap_or(default_fg);
            let modifier = weight_opt.map(parse_modifier).unwrap_or(default_mod);
            Style::default().fg(fg).add_modifier(modifier)
        };

        let notebook_item_style_normal = make_item_style(config.notebook_item_fg.as_deref(), config.notebook_item_weight.as_deref(), foreground, Modifier::empty());
        let notebook_item_style_selected = make_item_style(config.notebook_item_selected_fg.as_deref(), config.notebook_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);
        let notebook_icon_style_normal = make_item_style(config.notebook_icon_fg.as_deref(), config.notebook_item_weight.as_deref(), sidebar_title, Modifier::empty());
        let notebook_icon_style_selected = make_item_style(config.notebook_icon_selected_fg.as_deref().or(config.notebook_icon_fg.as_deref()), config.notebook_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);

        let section_item_style_normal = make_item_style(config.section_item_fg.as_deref(), config.section_item_weight.as_deref(), foreground, Modifier::empty());
        let section_item_style_selected = make_item_style(config.section_item_selected_fg.as_deref(), config.section_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);
        let section_icon_style_normal = make_item_style(config.section_icon_fg.as_deref(), config.section_item_weight.as_deref(), sidebar_title, Modifier::empty());
        let section_icon_style_selected = make_item_style(config.section_icon_selected_fg.as_deref().or(config.section_icon_fg.as_deref()), config.section_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);

        let note_item_style_normal = make_item_style(config.note_item_fg.as_deref(), config.note_item_weight.as_deref(), foreground, Modifier::empty());
        let note_item_style_selected = make_item_style(config.note_item_selected_fg.as_deref(), config.note_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);
        let note_icon_style_normal = make_item_style(config.note_icon_fg.as_deref(), config.note_item_weight.as_deref(), sidebar_title, Modifier::empty());
        let note_icon_style_selected = make_item_style(config.note_icon_selected_fg.as_deref().or(config.note_icon_fg.as_deref()), config.note_item_selected_weight.as_deref(), highlight_fg, Modifier::BOLD);

        Self {
            active_border,
            inactive_border,
            sidebar_title,
            active_sidebar_border,
            foreground,
            background,
            highlight_bg,
            highlight_fg,
            header_1: parse_color(&config.header_1),
            header_2: parse_color(&config.header_2),
            header_3: parse_color(&config.header_3),
            code_bg: parse_color(&config.code_bg),
            encrypted_tag: parse_color(&config.encrypted_tag),
            transparent,

            primary: active_border,
            secondary: sidebar_title,
            accent: active_sidebar_border,
            border: inactive_border,

            notebook_border_active: config.notebook_border_active.as_deref().map(parse_color).unwrap_or(active_sidebar_border),
            notebook_border_inactive: config.notebook_border_inactive.as_deref().map(parse_color).unwrap_or(inactive_border),
            notebook_title_active: config.notebook_title_active.as_deref().map(parse_color).unwrap_or(sidebar_title),
            notebook_title_inactive: config.notebook_title_inactive.as_deref().map(parse_color).unwrap_or(sidebar_title),

            section_border_active: config.section_border_active.as_deref().map(parse_color).unwrap_or(active_sidebar_border),
            section_border_inactive: config.section_border_inactive.as_deref().map(parse_color).unwrap_or(inactive_border),
            section_title_active: config.section_title_active.as_deref().map(parse_color).unwrap_or(sidebar_title),
            section_title_inactive: config.section_title_inactive.as_deref().map(parse_color).unwrap_or(sidebar_title),

            note_border_active: config.note_border_active.as_deref().map(parse_color).unwrap_or(active_sidebar_border),
            note_border_inactive: config.note_border_inactive.as_deref().map(parse_color).unwrap_or(inactive_border),
            note_title_active: config.note_title_active.as_deref().map(parse_color).unwrap_or(sidebar_title),
            note_title_inactive: config.note_title_inactive.as_deref().map(parse_color).unwrap_or(sidebar_title),

            preview_border_active: config.preview_border_active.as_deref().map(parse_color).unwrap_or(active_border),
            preview_border_inactive: config.preview_border_inactive.as_deref().map(parse_color).unwrap_or(inactive_border),
            preview_title_active: config.preview_title_active.as_deref().map(parse_color).unwrap_or(active_border),
            preview_title_inactive: config.preview_title_inactive.as_deref().map(parse_color).unwrap_or(sidebar_title),

            notebook_item_bg_normal,
            notebook_item_bg_selected,
            section_item_bg_normal,
            section_item_bg_selected,
            note_item_bg_normal,
            note_item_bg_selected,

            notebook_item_style_normal,
            notebook_item_style_selected,
            notebook_icon_style_normal,
            notebook_icon_style_selected,

            section_item_style_normal,
            section_item_style_selected,
            section_icon_style_normal,
            section_icon_style_selected,

            note_item_style_normal,
            note_item_style_selected,
            note_icon_style_normal,
            note_icon_style_selected,
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
        self.bg_style().fg(self.inactive_border)
    }

    pub fn active_main_border_style(&self) -> Style {
        self.bg_style().fg(self.active_border).add_modifier(Modifier::BOLD)
    }

    pub fn active_sidebar_border_style(&self) -> Style {
        self.bg_style().fg(self.active_sidebar_border).add_modifier(Modifier::BOLD)
    }

    pub fn sidebar_title_style(&self) -> Style {
        self.bg_style().fg(self.sidebar_title).add_modifier(Modifier::BOLD)
    }

    pub fn main_title_style(&self) -> Style {
        self.bg_style().fg(self.active_border).add_modifier(Modifier::BOLD)
    }

    pub fn active_border_style(&self) -> Style {
        self.active_main_border_style()
    }

    pub fn title_style(&self) -> Style {
        self.main_title_style()
    }

    pub fn active_title_style(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.active_border)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_active_style(&self) -> Style {
        Style::default()
            .bg(self.sidebar_title)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_inactive_style(&self) -> Style {
        self.bg_style().fg(self.inactive_border)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(self.highlight_fg)
            .add_modifier(Modifier::BOLD)
    }

    // Component-specific getters
    pub fn notebook_border_style(&self, active: bool) -> Style {
        let col = if active { self.notebook_border_active } else { self.notebook_border_inactive };
        self.bg_style().fg(col).add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    }

    pub fn notebook_title_style(&self, active: bool) -> Style {
        let col = if active { self.notebook_title_active } else { self.notebook_title_inactive };
        self.bg_style().fg(col).add_modifier(Modifier::BOLD)
    }

    pub fn notebook_item_style(&self, selected: bool) -> Style {
        if selected { self.notebook_item_style_selected } else { self.notebook_item_style_normal }
    }

    pub fn notebook_icon_style(&self, selected: bool) -> Style {
        if selected { self.notebook_icon_style_selected } else { self.notebook_icon_style_normal }
    }

    pub fn notebook_item_bg_style(&self, selected: bool) -> Style {
        let bg = if selected { self.notebook_item_bg_selected } else { self.notebook_item_bg_normal };
        if bg == Color::Reset { Style::default() } else { Style::default().bg(bg) }
    }

    pub fn section_border_style(&self, active: bool) -> Style {
        let col = if active { self.section_border_active } else { self.section_border_inactive };
        self.bg_style().fg(col).add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    }

    pub fn section_title_style_comp(&self, active: bool) -> Style {
        let col = if active { self.section_title_active } else { self.section_title_inactive };
        self.bg_style().fg(col).add_modifier(Modifier::BOLD)
    }

    pub fn section_item_style(&self, selected: bool) -> Style {
        if selected { self.section_item_style_selected } else { self.section_item_style_normal }
    }

    pub fn section_icon_style(&self, selected: bool) -> Style {
        if selected { self.section_icon_style_selected } else { self.section_icon_style_normal }
    }

    pub fn section_item_bg_style(&self, selected: bool) -> Style {
        let bg = if selected { self.section_item_bg_selected } else { self.section_item_bg_normal };
        if bg == Color::Reset { Style::default() } else { Style::default().bg(bg) }
    }

    pub fn note_border_style(&self, active: bool) -> Style {
        let col = if active { self.note_border_active } else { self.note_border_inactive };
        self.bg_style().fg(col).add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    }

    pub fn note_title_style(&self, active: bool) -> Style {
        let col = if active { self.note_title_active } else { self.note_title_inactive };
        self.bg_style().fg(col).add_modifier(Modifier::BOLD)
    }

    pub fn note_item_style(&self, selected: bool) -> Style {
        if selected { self.note_item_style_selected } else { self.note_item_style_normal }
    }

    pub fn note_icon_style(&self, selected: bool) -> Style {
        if selected { self.note_icon_style_selected } else { self.note_icon_style_normal }
    }

    pub fn note_item_bg_style(&self, selected: bool) -> Style {
        let bg = if selected { self.note_item_bg_selected } else { self.note_item_bg_normal };
        if bg == Color::Reset { Style::default() } else { Style::default().bg(bg) }
    }

    pub fn preview_border_style(&self, active: bool) -> Style {
        let col = if active { self.preview_border_active } else { self.preview_border_inactive };
        self.bg_style().fg(col).add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    }

    pub fn preview_title_style(&self, active: bool) -> Style {
        let col = if active { self.preview_title_active } else { self.preview_title_inactive };
        self.bg_style().fg(col).add_modifier(Modifier::BOLD)
    }
}

pub fn parse_modifier(modifier_str: &str) -> Modifier {
    let mut modifier = Modifier::empty();
    let lower = modifier_str.to_lowercase();
    for part in lower.split(|c| c == '|' || c == ',' || c == '+' || c == ' ') {
        match part.trim() {
            "bold" => modifier |= Modifier::BOLD,
            "dim" => modifier |= Modifier::DIM,
            "italic" => modifier |= Modifier::ITALIC,
            "underlined" | "underline" => modifier |= Modifier::UNDERLINED,
            "reversed" | "reverse" => modifier |= Modifier::REVERSED,
            "crossed_out" | "strikethrough" => modifier |= Modifier::CROSSED_OUT,
            "normal" | "regular" | "none" => {}
            _ => {}
        }
    }
    modifier
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_builtin_themes_deserialize_validly() {
        for (name, content) in BUILTIN_THEMES {
            let res = toml::from_str::<ThemeConfig>(content);
            assert!(
                res.is_ok(),
                "Failed to deserialize built-in theme {}: {:?}",
                name,
                res.err()
            );
        }
    }

    #[test]
    fn test_load_theme_by_name_presets() {
        let nord = load_theme_by_name("nord");
        assert_eq!(nord.active_border, "#88C0D0");

        let mocha = load_theme_by_name("catppuccin");
        assert_eq!(mocha.active_border, "#CBA6F7");

        let mocha_explicit = load_theme_by_name("catppuccin-mocha");
        assert_eq!(mocha_explicit.active_border, "#CBA6F7");

        let dracula = load_theme_by_name("dracula");
        assert_eq!(dracula.active_border, "#BD93F9");

        let gruvbox = load_theme_by_name("gruvbox");
        assert_eq!(gruvbox.active_border, "#FABD2F");

        let tokyo = load_theme_by_name("tokyo-night");
        assert_eq!(tokyo.active_border, "#7AA2F7");

        let ayu = load_theme_by_name("ayu-dark");
        assert_eq!(ayu.active_border, "#FFCC66");

        let solarized = load_theme_by_name("solarized-dark");
        assert_eq!(solarized.active_border, "#268BD2");

        let monokai = load_theme_by_name("monokai");
        assert_eq!(monokai.active_border, "#FD971F");
    }

    #[test]
    fn test_init_themes_dir() {
        let temp_dir = std::env::temp_dir().join(format!("notedog_test_{}", rand::random::<u64>()));
        init_themes_dir(&temp_dir);
        let themes_path = temp_dir.join("themes");
        assert!(themes_path.exists());
        for (filename, _) in BUILTIN_THEMES {
            assert!(themes_path.join(filename).exists(), "Missing generated theme file: {}", filename);
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}


