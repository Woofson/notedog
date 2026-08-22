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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    Notebook,
    Section,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconRule {
    pub pattern: String,
    pub icon: String,
    #[serde(default, alias = "scope", alias = "type", alias = "applies_to")]
    pub target: Option<String>,
}

impl IconRule {
    pub fn matches(&self, name: &str) -> bool {
        if let Ok(re) = regex::Regex::new(&self.pattern) {
            if re.is_match(name) {
                return true;
            }
        } else if name.to_lowercase().contains(&self.pattern.to_lowercase()) {
            return true;
        }
        false
    }
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
    pub fn get_icon_for(&self, name: &str, item_type: IconType, default_fallback: &str) -> String {
        // 1. Check specific target rules first (e.g. target = "section", "note", "notebook")
        for rule in &self.rules {
            if let Some(target) = &rule.target {
                let t = target.trim().to_lowercase();
                let matches_target = match item_type {
                    IconType::Notebook => t == "notebook" || t == "notebooks" || t == "nb",
                    IconType::Section => t == "section" || t == "sections" || t == "sec",
                    IconType::Note => t == "note" || t == "notes",
                };
                if matches_target && rule.matches(name) {
                    return rule.icon.clone();
                }
            }
        }

        // 2. Check generic rules (target is None or "all")
        for rule in &self.rules {
            let is_generic = match &rule.target {
                None => true,
                Some(t) => {
                    let s = t.trim().to_lowercase();
                    s.is_empty() || s == "all" || s == "*" || s == "any"
                }
            };
            if is_generic && rule.matches(name) {
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
            rules: vec![
                IconRule { pattern: "(?i).*(todo|tasks|tasklist|checklist|to-do).*".to_string(), icon: "✅ ".to_string(), target: None },
                IconRule { pattern: "(?i).*(shopping|grocery|groceries|store|buy|buy-list).*".to_string(), icon: "🛒 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(idea|ideas|brainstorm|concept).*".to_string(), icon: "💡 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(work|job|office|project|sprint).*".to_string(), icon: "💼 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(personal|journal|diary|daily).*".to_string(), icon: "📔 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(finance|budget|money|expense|expenses|bank).*".to_string(), icon: "💰 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(secret|secrets|passwords|vault|private).*".to_string(), icon: "🔒 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(meeting|meetings|call|agenda|standup).*".to_string(), icon: "📅 ".to_string(), target: None },
                IconRule { pattern: "(?i).*(welcome|intro|getting-started|readme).*".to_string(), icon: "👋 ".to_string(), target: None },
            ],
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
pub struct TemplateRule {
    pub pattern: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemeSetting {
    Name(String),
    Inline(ThemeConfig),
}

impl Default for ThemeSetting {
    fn default() -> Self {
        ThemeSetting::Name("notedog".to_string())
    }
}

fn default_notebooks_title() -> String { "NOTEBOOKS".to_string() }
fn default_sections_title() -> String { "SECTIONS".to_string() }
fn default_notes_title() -> String { "NOTES".to_string() }
fn default_show_main_title() -> bool { false }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitlesConfig {
    #[serde(default = "default_notebooks_title")]
    pub notebooks: String,
    #[serde(default = "default_sections_title")]
    pub sections: String,
    #[serde(default = "default_notes_title")]
    pub notes: String,
    #[serde(default = "default_show_main_title")]
    pub show_main_title: bool,
}

impl Default for TitlesConfig {
    fn default() -> Self {
        Self {
            notebooks: default_notebooks_title(),
            sections: default_sections_title(),
            notes: default_notes_title(),
            show_main_title: default_show_main_title(),
        }
    }
}

fn default_spawn_examples() -> bool { true }
fn default_spawn_themes() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub note_folder: String,
    pub editor: String,
    pub secrets_file: String,
    pub transparent_background: bool,
    pub show_help_bar: bool,
    pub word_wrap: bool,
    pub default_notebook: String,

    #[serde(default = "default_spawn_examples", alias = "spawn_example_files", alias = "generate_examples")]
    pub spawn_examples: bool,

    #[serde(default = "default_spawn_themes", alias = "spawn_builtin_themes", alias = "auto_create_themes")]
    pub spawn_themes: bool,

    #[serde(default)]
    pub theme: ThemeSetting,

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
    pub titles: TitlesConfig,

    #[serde(default)]
    pub icons: IconConfig,

    #[serde(default)]
    pub templates: Vec<TemplateRule>,
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
            spawn_examples: default_spawn_examples(),
            spawn_themes: default_spawn_themes(),
            theme: ThemeSetting::default(),
            default_note_prefix: default_note_prefix(),
            default_note_postfix: default_note_postfix(),
            default_section_prefix: default_section_prefix(),
            default_section_postfix: default_section_postfix(),
            date_format: default_date_format(),
            layout: LayoutConfig::default(),
            titles: TitlesConfig::default(),
            icons: IconConfig::default(),
            templates: Vec::new(),
        }
    }
}

impl Config {
    pub fn get_template_for(&self, title: &str, date_str: &str) -> String {
        // 1. Check custom template rules from config first
        for rule in &self.templates {
            if let Ok(re) = regex::Regex::new(&rule.pattern) {
                if re.is_match(title) {
                    return rule.template
                        .replace("{{title}}", title)
                        .replace("{{date}}", date_str);
                }
            } else if title.to_lowercase().contains(&rule.pattern.to_lowercase()) {
                return rule.template
                    .replace("{{title}}", title)
                    .replace("{{date}}", date_str);
            }
        }

        // 2. Built-in Preset Note Templates
        let lower = title.to_lowercase();

        // Todo / Task List Template
        if lower.contains("todo") || lower.contains("tasks") || lower.contains("tasklist") || lower.contains("checklist") || lower.contains("to-do") {
            return format!(
                "# 📝 {}\n\n**Created**: {}\n\n- [ ] Task 1\n- [ ] Task 2\n- [ ] Task 3\n",
                title, date_str
            );
        }

        // Shopping / Grocery List Template
        if lower.contains("shopping") || lower.contains("grocery") || lower.contains("groceries") || lower.contains("buy") {
            return format!(
                "# 🛒 {}\n\n**Created**: {}\n\n## 🥦 Produce & Fresh Food\n- [ ] Apples\n- [ ] Milk\n\n## 🥛 Dairy & Pantry\n- [ ] Bread\n- [ ] Coffee\n\n## 🧼 Household & Personal Care\n- [ ] Paper Towels\n",
                title, date_str
            );
        }

        // Meetings / Standup / Agenda Template
        if lower.contains("meeting") || lower.contains("call") || lower.contains("agenda") || lower.contains("standup") {
            return format!(
                "# 📅 {}\n\n**Date**: {}\n**Attendees**: \n\n## 🎯 Agenda\n1. Topic 1\n2. Topic 2\n\n## 📝 Discussion & Notes\n- Key point 1\n\n## ⚡ Action Items\n- [ ] Action item 1\n",
                title, date_str
            );
        }

        // Ideas / Brainstorming Template
        if lower.contains("idea") || lower.contains("brainstorm") || lower.contains("concept") {
            return format!(
                "# 💡 {}\n\n**Created**: {}\n\n## 🎯 Core Concept\n\n## 🚀 Potential Impact & Goals\n\n## 📝 Next Steps\n- [ ] Research & prototype\n",
                title, date_str
            );
        }

        // Work / Sprint / Project Template
        if lower.contains("work") || lower.contains("project") || lower.contains("sprint") || lower.contains("job") {
            return format!(
                "# 💼 {}\n\n**Created**: {}\n\n## 📌 Overview\n\n## 🎯 Objectives\n- [ ] Objective 1\n\n## 📝 Status & Updates\n",
                title, date_str
            );
        }

        // Finance / Budget / Expenses Template
        if lower.contains("finance") || lower.contains("budget") || lower.contains("money") || lower.contains("expense") {
            return format!(
                "# 💰 {}\n\n**Created**: {}\n\n## 📊 Summary\n- **Income**: $0.00\n- **Expenses**: $0.00\n\n## 📝 Expense Breakdown\n- [ ] Fixed Expenses\n",
                title, date_str
            );
        }

        // Default Regular Note Template
        format!(
            "# {}\n\n**Created**: {}\n\n- Write your note content here...\n",
            title, date_str
        )
    }

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

    pub fn load_theme(&self) -> ThemeConfig {
        match &self.theme {
            ThemeSetting::Name(name) => crate::theme::load_theme_by_name(name),
            ThemeSetting::Inline(tc) => tc.clone(),
        }
    }

    pub fn theme_name(&self) -> &str {
        match &self.theme {
            ThemeSetting::Name(name) => name.as_str(),
            ThemeSetting::Inline(_) => "custom",
        }
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

        let (cfg, target_path) = if toml_path.exists() {
            let content = fs::read_to_string(&toml_path).unwrap_or_default();
            let parsed = toml::from_str::<Config>(&content).unwrap_or_default();
            (parsed, toml_path)
        } else if conf_path.exists() {
            let content = fs::read_to_string(&conf_path).unwrap_or_default();
            let parsed = toml::from_str::<Config>(&content).unwrap_or_default();
            (parsed, conf_path)
        } else {
            let default_cfg = Config::default();
            let content = toml::to_string_pretty(&default_cfg).unwrap_or_default();
            let comment_header = r#"# Notedog Configuration File
# Location: ~/.config/notedog/notedog.toml or ~/.config/notedog/notedog.conf
# See ~/.config/notedog/notedog.toml.example for full options & comments
# Themes are stored in ~/.config/notedog/themes/<theme>.toml

"#;
            let full_content = format!("{}{}", comment_header, content);
            let _ = fs::write(&toml_path, full_content);
            (default_cfg, toml_path)
        };

        // If spawn_themes is enabled (default: true), populate ~/.config/notedog/themes/ with built-in presets
        if cfg.spawn_themes {
            crate::theme::init_themes_dir(&config_dir);
        }

        // If spawn_examples is enabled (default: true), write .example files if they do not exist
        if cfg.spawn_examples {
            if !example_cfg_path.exists() {
                let _ = fs::write(&example_cfg_path, include_str!("../notedog.toml.example"));
            }
            if !example_theme_path.exists() {
                let _ = fs::write(&example_theme_path, include_str!("../theme.toml.example"));
            }
        }

        (cfg, target_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_theme_name_deserialization() {
        let toml_str = r#"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"
            theme = "nord"
        "#;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize config");
        assert_eq!(cfg.theme_name(), "nord");
        let tc = cfg.load_theme();
        assert_eq!(tc.active_border, "#88C0D0");
    }

    #[test]
    fn test_config_legacy_inline_theme() {
        let toml_str = r##"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"

            [theme]
            active_border = "#123456"
        "##;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize legacy config");
        assert_eq!(cfg.theme_name(), "custom");
        let tc = cfg.load_theme();
        assert_eq!(tc.active_border, "#123456");
    }

    #[test]
    fn test_config_default_theme() {
        let cfg = Config::default();
        assert_eq!(cfg.theme_name(), "notedog");
        let tc = cfg.load_theme();
        assert_eq!(tc.active_border, "#FFCC66");
    }

    #[test]
    fn test_config_titles_customization() {
        let toml_str = r#"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"

            [titles]
            notebooks = "My Books"
            sections = "My Chapters"
            notes = "My Pages"
            show_main_title = true
        "#;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize config");
        assert_eq!(cfg.titles.notebooks, "My Books");
        assert_eq!(cfg.titles.sections, "My Chapters");
        assert_eq!(cfg.titles.notes, "My Pages");
        assert_eq!(cfg.titles.show_main_title, true);
    }

    #[test]
    fn test_scoped_icon_rules() {
        let toml_str = r#"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"

            [icons]
            notebook = "📚 "
            section = "📂 "
            note = "📄 "

            [[icons.rules]]
            pattern = "(?i).*groceries.*"
            target = "notebook"
            icon = "🏢 "

            [[icons.rules]]
            pattern = "(?i).*groceries.*"
            target = "section"
            icon = "🛒 "

            [[icons.rules]]
            pattern = "(?i).*groceries.*"
            target = "note"
            icon = "🥦 "
        "#;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize config");
        assert_eq!(cfg.icons.get_icon_for("Groceries", IconType::Notebook, &cfg.icons.notebook), "🏢 ");
        assert_eq!(cfg.icons.get_icon_for("Groceries", IconType::Section, &cfg.icons.section), "🛒 ");
        assert_eq!(cfg.icons.get_icon_for("Groceries", IconType::Note, &cfg.icons.note), "🥦 ");

        // Also test generic fallback rule if target is not specified
        let toml_str_generic = r#"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"

            [icons]
            notebook = "📚 "
            section = "📂 "
            note = "📄 "

            [[icons.rules]]
            pattern = "(?i).*groceries.*"
            target = "section"
            icon = "🛒 "

            [[icons.rules]]
            pattern = "(?i).*groceries.*"
            icon = "📝 "
        "#;
        let cfg_generic: Config = toml::from_str(toml_str_generic).expect("failed to deserialize config");
        assert_eq!(cfg_generic.icons.get_icon_for("Groceries", IconType::Section, &cfg_generic.icons.section), "🛒 ");
        assert_eq!(cfg_generic.icons.get_icon_for("Groceries", IconType::Notebook, &cfg_generic.icons.notebook), "📝 ");
        assert_eq!(cfg_generic.icons.get_icon_for("Groceries", IconType::Note, &cfg_generic.icons.note), "📝 ");
    }

    #[test]
    fn test_config_spawn_examples_and_themes() {
        let toml_str = r#"
            note_folder = "~/Notes"
            editor = "builtin"
            secrets_file = "~/.config/notedog/secrets.toml"
            transparent_background = true
            show_help_bar = true
            word_wrap = true
            default_notebook = "Personal"
            spawn_examples = false
            spawn_themes = false
        "#;
        let cfg: Config = toml::from_str(toml_str).expect("failed to deserialize config");
        assert_eq!(cfg.spawn_examples, false);
        assert_eq!(cfg.spawn_themes, false);

        let default_cfg = Config::default();
        assert_eq!(default_cfg.spawn_examples, true);
        assert_eq!(default_cfg.spawn_themes, true);
    }
}




