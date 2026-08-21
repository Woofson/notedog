use crate::theme::ThemeConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use ratatui::layout::Constraint;

fn default_note_prefix() -> String { "Note ".to_string() }
fn default_note_postfix() -> String { "".to_string() }
fn default_section_prefix() -> String { "Section ".to_string() }
fn default_section_postfix() -> String { "".to_string() }
fn default_date_format() -> String { "%Y-%m-%d %H:%M".to_string() }

fn default_notebook_icon() -> String { "📚 ".to_string() }
fn default_section_icon() -> String { "📂 ".to_string() }
fn default_note_icon() -> String { "📄 ".to_string() }
fn default_encrypted_note_icon() -> String { "🔒 ".to_string() }
fn default_preview_icon() -> String { "📖 ".to_string() }
fn default_editor_icon() -> String { "✏️ ".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconRule {
    pub pattern: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconConfig {
    #[serde(default = "default_notebook_icon")]
    pub notebook: String,
    #[serde(default = "default_section_icon")]
    pub section: String,
    #[serde(default = "default_note_icon")]
    pub note: String,
    #[serde(default = "default_encrypted_note_icon")]
    pub encrypted_note: String,
    #[serde(default = "default_preview_icon")]
    pub preview: String,
    #[serde(default = "default_editor_icon")]
    pub editor: String,
    #[serde(default)]
    pub rules: Vec<IconRule>,
}

impl IconConfig {
    pub fn get_icon_for(&self, name: &str, default_fallback: &str) -> String {
        for rule in &self.rules {
            if let Ok(re) = regex::Regex::new(&rule.pattern) {
                if re.is_match(name) {
                    return rule.icon.clone();
                }
            } else if name.contains(&rule.pattern) {
                return rule.icon.clone();
            }
        }
        default_fallback.to_string()
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            notebook: default_notebook_icon(),
            section: default_section_icon(),
            note: default_note_icon(),
            encrypted_note: default_encrypted_note_icon(),
            preview: default_preview_icon(),
            editor: default_editor_icon(),
            rules: Vec::new(),
        }
    }
}

fn default_sidebar_width() -> String { "26%".to_string() }
fn default_notebooks_height() -> String { "26%".to_string() }
fn default_sections_height() -> String { "34%".to_string() }
fn default_notes_height() -> String { "40%".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: String,
    #[serde(default = "default_notebooks_height")]
    pub notebooks_height: String,
    #[serde(default = "default_sections_height")]
    pub sections_height: String,
    #[serde(default = "default_notes_height")]
    pub notes_height: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: default_sidebar_width(),
            notebooks_height: default_notebooks_height(),
            sections_height: default_sections_height(),
            notes_height: default_notes_height(),
        }
    }
}

pub fn parse_constraint(s: &str, default_pct: u16) -> Constraint {
    let trimmed = s.trim();
    if trimmed.ends_with('%') {
        if let Ok(pct) = trimmed[..trimmed.len() - 1].parse::<u16>() {
            return Constraint::Percentage(pct);
        }
    } else if let Ok(val) = trimmed.parse::<u16>() {
        return Constraint::Length(val);
    }
    Constraint::Percentage(default_pct)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub note_folder: String,
    pub editor: String,
    pub secrets_file: String,
    pub transparent_background: bool,
    pub show_help_bar: bool,
    pub word_wrap: bool,
    pub default_notebook: String,

    #[serde(default = "default_note_prefix")]
    pub default_note_prefix: String,
    #[serde(default = "default_note_postfix")]
    pub default_note_postfix: String,
    #[serde(default = "default_section_prefix")]
    pub default_section_prefix: String,
    #[serde(default = "default_section_postfix")]
    pub default_section_postfix: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,

    #[serde(default)]
    pub layout: LayoutConfig,

    #[serde(default)]
    pub icons: IconConfig,

    #[serde(default)]
    pub theme: ThemeConfig,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let note_folder = home.join("Notes").to_string_lossy().to_string();
        let secrets_file = home
            .join(".config")
            .join("notedog")
            .join("secrets.toml")
            .to_string_lossy()
            .to_string();

        Self {
            note_folder,
            editor: "builtin".to_string(), // "builtin", "nvim", "nano", "micro"
            secrets_file,
            transparent_background: true,
            show_help_bar: true,
            word_wrap: true,
            default_notebook: "Personal".to_string(),
            default_note_prefix: default_note_prefix(),
            default_note_postfix: default_note_postfix(),
            default_section_prefix: default_section_prefix(),
            default_section_postfix: default_section_postfix(),
            date_format: default_date_format(),
            layout: LayoutConfig::default(),
            icons: IconConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Config {
    pub fn format_default_note_title(&self) -> String {
        let ts = crate::versioning::format_timestamp(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        format!("{}{}{}", self.default_note_prefix, ts, self.default_note_postfix)
    }

    pub fn format_default_section_title(&self) -> String {
        let ts = crate::versioning::format_timestamp(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        format!("{}{}{}", self.default_section_prefix, ts, self.default_section_postfix)
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("notedog")
    }

    pub fn load_or_create() -> (Self, PathBuf) {
        let config_dir = Self::config_dir();
        let toml_path = config_dir.join("notedog.toml");
        let conf_path = config_dir.join("notedog.conf");

        let example_cfg_path = config_dir.join("notedog.toml.example");
        let example_theme_path = config_dir.join("theme.toml.example");

        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Warning: could not create config dir {:?}: {}", config_dir, e);
        }

        if !example_cfg_path.exists() {
            let _ = fs::write(&example_cfg_path, include_str!("../notedog.toml.example"));
        }
        if !example_theme_path.exists() {
            let _ = fs::write(&example_theme_path, include_str!("../theme.toml.example"));
        }

        let target_path = if toml_path.exists() {
            toml_path
        } else if conf_path.exists() {
            conf_path
        } else {
            let default_cfg = Config::default();
            let content = toml::to_string_pretty(&default_cfg).unwrap_or_default();
            let comment_header = r#"# Notedog Configuration File
# Location: ~/.config/notedog/notedog.toml or ~/.config/notedog/notedog.conf
# See ~/.config/notedog/notedog.toml.example for full options & comments

"#;
            let full_content = format!("{}{}", comment_header, content);
            let _ = fs::write(&toml_path, full_content);
            toml_path
        };

        if let Ok(content) = fs::read_to_string(&target_path) {
            if let Ok(cfg) = toml::from_str::<Config>(&content) {
                return (cfg, target_path);
            }
        }

        (Config::default(), target_path)
    }

    pub fn resolved_note_folder(&self) -> PathBuf {
        expand_path(&self.note_folder)
    }

    pub fn resolved_secrets_file(&self) -> PathBuf {
        expand_path(&self.secrets_file)
    }
}

pub fn expand_path(p: &str) -> PathBuf {
    if p.starts_with("~/") || p == "~" {
        if let Some(home) = dirs::home_dir() {
            if p == "~" {
                return home;
            } else {
                return home.join(&p[2..]);
            }
        }
    }
    PathBuf::from(p)
}
