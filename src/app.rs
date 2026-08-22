use crate::config::Config;
use crate::crypto::{decrypt_note, encrypt_note, is_encrypted_data};
use crate::editor::Editor;
use crate::note_manager::{NoteFile, NoteManager};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Notebooks,
    Sections,
    Notes,
    MainView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Preview,
    Editor,
}

use crate::versioning::{VersionInfo, VersionManager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    PassphrasePrompt {
        prompt: String,
        label: String,
        error: Option<String>,
    },
    CreateNotebook,
    CreateSection,
    CreateNote,
    RenameNotebook,
    RenameSection,
    RenameNote,
    ConfirmDelete {
        title: String,
        item_type: String,
    },
    ConfirmEditorExit,
    VersionHistory,
    VersionCleanup,
    About,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingCryptoAction {
    UnlockNote,
    EncryptCurrentNote { first_pass: Option<String> },
    DecryptCurrentNote,
    EncryptCurrentSection { first_pass: Option<String> },
    DecryptCurrentSection,
    EncryptCurrentNotebook { first_pass: Option<String> },
    DecryptCurrentNotebook,
    ChangePassword { current_pass: Option<String>, new_pass: Option<String> },
}

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub manager: NoteManager,
    pub version_manager: VersionManager,

    pub active_notebook_idx: usize,
    pub active_section_idx: usize,
    pub active_note_idx: usize,

    pub focused_pane: Pane,
    pub view_mode: ViewMode,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub pending_action: Option<PendingCryptoAction>,

    pub current_versions: Vec<VersionInfo>,
    pub selected_version_idx: usize,
    pub selected_preset_idx: usize,

    pub editor: Editor,
    pub preview_scroll_y: usize,
    pub help_scroll_y: usize,
    pub diff_scroll_y: usize,
    pub is_fullscreen: bool,
    pub word_wrap: bool,
    pub show_help: bool,
    pub needs_clear: bool,
    pub status_message: String,
    pub current_note_content: String,
    pub cached_passphrase: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let theme = Theme::load_for_config(&config);
        let note_folder = config.resolved_note_folder();
        let manager = NoteManager::new(note_folder.clone());
        let version_manager = VersionManager::new(&note_folder);

        let mut app = Self {
            word_wrap: config.word_wrap,
            config,
            theme,
            manager,
            version_manager,
            active_notebook_idx: 0,
            active_section_idx: 0,
            active_note_idx: 0,
            focused_pane: Pane::Notes,
            view_mode: ViewMode::Preview,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            pending_action: None,
            current_versions: Vec::new(),
            selected_version_idx: 0,
            selected_preset_idx: 0,
            editor: Editor::default(),
            preview_scroll_y: 0,
            help_scroll_y: 0,
            diff_scroll_y: 0,
            is_fullscreen: false,
            show_help: false,
            needs_clear: false,
            status_message: "Ready".to_string(),
            current_note_content: String::new(),
            cached_passphrase: None,
        };

        app.load_current_note();
        app
    }

    pub fn current_note_file(&self) -> Option<&NoteFile> {
        self.manager
            .notebooks
            .get(self.active_notebook_idx)
            .and_then(|nb| nb.sections.get(self.active_section_idx))
            .and_then(|sec| sec.notes.get(self.active_note_idx))
    }

    pub fn current_notebook_name(&self) -> String {
        self.manager
            .notebooks
            .get(self.active_notebook_idx)
            .map(|nb| nb.name.clone())
            .unwrap_or_else(|| "None".to_string())
    }

    pub fn current_section_name(&self) -> String {
        self.manager
            .notebooks
            .get(self.active_notebook_idx)
            .and_then(|nb| nb.sections.get(self.active_section_idx))
            .map(|sec| sec.name.clone())
            .unwrap_or_else(|| "None".to_string())
    }

    pub fn current_note_title(&self) -> String {
        self.current_note_file()
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "No Note Selected".to_string())
    }

    pub fn is_current_note_encrypted(&self) -> bool {
        self.current_note_file()
            .map(|n| n.is_encrypted)
            .unwrap_or(false)
    }

    pub fn current_note_dir(&self) -> Option<std::path::PathBuf> {
        self.current_note_file().and_then(|n| n.path.parent().map(|p| p.to_path_buf()))
    }

    pub fn load_current_note(&mut self) {
        self.preview_scroll_y = 0;
        let note_file = match self.current_note_file() {
            Some(n) => n.clone(),
            None => {
                self.current_note_content = "# No Note Selected\n\nCreate a new note with `Ctrl+N`.".to_string();
                self.editor = Editor::from_string(&self.current_note_content);
                return;
            }
        };

        if let Ok(raw_bytes) = self.manager.read_note_raw(&note_file.path) {
            let _ = self.version_manager.create_snapshot(&note_file.path, &raw_bytes);
            if note_file.is_encrypted || is_encrypted_data(&raw_bytes) {
                if let Some(pass) = &self.cached_passphrase {
                    match decrypt_note(&raw_bytes, pass) {
                        Ok(decrypted) => {
                            self.current_note_content = decrypted.clone();
                            self.editor = Editor::from_string(&decrypted);
                            self.status_message = "Unlocked encrypted note".to_string();
                        }
                        Err(_) => {
                            self.current_note_content = "🔒 [Encrypted Note - Enter Passphrase to view]\nPress Enter to unlock.".to_string();
                            self.editor = Editor::from_string(&self.current_note_content);
                            self.status_message = "Passphrase required".to_string();
                        }
                    }
                } else {
                    self.current_note_content = "🔒 [Encrypted Note - Enter Passphrase to view]\nPress Enter to unlock.".to_string();
                    self.editor = Editor::from_string(&self.current_note_content);
                    self.status_message = "Encrypted note. Press Enter to unlock.".to_string();
                }
            } else {
                let content = String::from_utf8_lossy(&raw_bytes).to_string();
                self.current_note_content = content.clone();
                self.editor = Editor::from_string(&content);
                self.status_message = format!("Loaded {}", note_file.name);
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                    self.show_help = false;
                    self.help_scroll_y = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll_y = self.help_scroll_y.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll_y += 1;
                }
                KeyCode::PageUp => {
                    self.help_scroll_y = self.help_scroll_y.saturating_sub(5);
                }
                KeyCode::PageDown => {
                    self.help_scroll_y += 5;
                }
                _ => {}
            }
            return false;
        }

        match &self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::PassphrasePrompt { .. } => self.handle_passphrase_mode(key),
            InputMode::CreateNotebook => self.handle_input_dialog(key, "notebook"),
            InputMode::CreateSection => self.handle_input_dialog(key, "section"),
            InputMode::CreateNote => self.handle_input_dialog(key, "note"),
            InputMode::RenameNotebook => self.handle_input_dialog(key, "rename_notebook"),
            InputMode::RenameSection => self.handle_input_dialog(key, "rename_section"),
            InputMode::RenameNote => self.handle_input_dialog(key, "rename_note"),
            InputMode::ConfirmDelete { title, item_type } => {
                let t = title.clone();
                let it = item_type.clone();
                self.handle_confirm_delete(key, &t, &it)
            }
            InputMode::ConfirmEditorExit => self.handle_confirm_editor_exit(key),
            InputMode::VersionHistory => self.handle_version_history(key),
            InputMode::VersionCleanup => self.handle_version_cleanup(key),
            InputMode::About => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::F(2) {
                    self.input_mode = InputMode::Normal;
                }
                false
            }
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        // Toggle About page with F2 or Ctrl+A
        if key.code == KeyCode::F(2) || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('a')) {
            self.input_mode = InputMode::About;
            return false;
        }

        // Toggle fullscreen with F11 or Ctrl+F
        if key.code == KeyCode::F(11) || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('f')) {
            self.is_fullscreen = !self.is_fullscreen;
            if self.is_fullscreen {
                self.focused_pane = Pane::MainView;
                self.status_message = "Entered Fullscreen mode".to_string();
            } else {
                self.status_message = "Exited Fullscreen mode".to_string();
            }
            return false;
        }

        // Contextual Delete (Ctrl+D or 'd' in preview)
        if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('d'))
            || (self.view_mode == ViewMode::Preview && key.code == KeyCode::Char('d'))
        {
            match self.focused_pane {
                Pane::Notebooks => {
                    if let Some(nb) = self.manager.notebooks.get(self.active_notebook_idx) {
                        self.input_mode = InputMode::ConfirmDelete {
                            title: nb.name.clone(),
                            item_type: "notebook".to_string(),
                        };
                    }
                    return false;
                }
                Pane::Sections => {
                    if let Some(nb) = self.manager.notebooks.get(self.active_notebook_idx) {
                        if let Some(sec) = nb.sections.get(self.active_section_idx) {
                            self.input_mode = InputMode::ConfirmDelete {
                                title: sec.name.clone(),
                                item_type: "section".to_string(),
                            };
                        }
                    }
                    return false;
                }
                Pane::Notes | Pane::MainView => {
                    if let Some(note) = self.current_note_file() {
                        self.input_mode = InputMode::ConfirmDelete {
                            title: note.name.clone(),
                            item_type: "note".to_string(),
                        };
                    }
                    return false;
                }
            }
        }

        // Contextual Rename (r in preview or Ctrl+R)
        if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r'))
            || (self.view_mode == ViewMode::Preview && key.code == KeyCode::Char('r'))
        {
            match self.focused_pane {
                Pane::Notebooks => {
                    if let Some(nb) = self.manager.notebooks.get(self.active_notebook_idx) {
                        self.input_buffer = nb.name.clone();
                        self.input_mode = InputMode::RenameNotebook;
                    }
                    return false;
                }
                Pane::Sections => {
                    if let Some(nb) = self.manager.notebooks.get(self.active_notebook_idx) {
                        if let Some(sec) = nb.sections.get(self.active_section_idx) {
                            self.input_buffer = sec.name.clone();
                            self.input_mode = InputMode::RenameSection;
                        }
                    }
                    return false;
                }
                Pane::Notes | Pane::MainView => {
                    if let Some(note) = self.current_note_file() {
                        self.input_buffer = note.name.clone();
                        self.input_mode = InputMode::RenameNote;
                    }
                    return false;
                }
            }
        }

        // Open Version History Modal (v or Ctrl+V or Ctrl+H)
        if (key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('v') || key.code == KeyCode::Char('h')))
            || (self.view_mode == ViewMode::Preview && key.code == KeyCode::Char('v'))
        {
            if let Some(note) = self.current_note_file() {
                self.current_versions = self.version_manager.list_versions(&note.path).unwrap_or_default();
                self.selected_version_idx = 0;
                self.input_mode = InputMode::VersionHistory;
            }
            return false;
        }

        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('n') => {
                    match self.focused_pane {
                        Pane::Notebooks => self.input_mode = InputMode::CreateNotebook,
                        Pane::Sections => self.input_mode = InputMode::CreateSection,
                        _ => self.input_mode = InputMode::CreateNote,
                    }
                    self.input_buffer.clear();
                    return false;
                }
                KeyCode::Char('b') => {
                    self.input_mode = InputMode::CreateNotebook;
                    self.input_buffer.clear();
                    return false;
                }
                KeyCode::Char('k') => {
                    self.input_mode = InputMode::CreateSection;
                    self.input_buffer.clear();
                    return false;
                }
                KeyCode::Char('p') => {
                    // Change Passphrase for focused Notebook, Section, or Note
                    match self.focused_pane {
                        Pane::Notebooks => {
                            let nb_name = self.current_notebook_name();
                            let is_enc = self.manager.notebooks.get(self.active_notebook_idx).map(|nb| nb.is_encrypted).unwrap_or(false);
                            if !is_enc {
                                self.status_message = format!("Notebook '{}' is not encrypted. Use Ctrl+E to encrypt.", nb_name);
                                return false;
                            }
                            self.pending_action = Some(PendingCryptoAction::ChangePassword { current_pass: None, new_pass: None });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Change Passphrase: {}", nb_name),
                                label: "Enter Current Passphrase:".to_string(),
                                error: None,
                            };
                        }
                        Pane::Sections => {
                            let sec_name = self.current_section_name();
                            let is_enc = self.manager.notebooks.get(self.active_notebook_idx)
                                .and_then(|nb| nb.sections.get(self.active_section_idx))
                                .map(|sec| sec.is_encrypted)
                                .unwrap_or(false);
                            if !is_enc {
                                self.status_message = format!("Section '{}' is not encrypted. Use Ctrl+E to encrypt.", sec_name);
                                return false;
                            }
                            self.pending_action = Some(PendingCryptoAction::ChangePassword { current_pass: None, new_pass: None });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Change Passphrase: {}", sec_name),
                                label: "Enter Current Passphrase:".to_string(),
                                error: None,
                            };
                        }
                        Pane::Notes | Pane::MainView => {
                            let note_title = self.current_note_title();
                            if !self.is_current_note_encrypted() {
                                self.status_message = format!("Note '{}' is not encrypted. Use Ctrl+E to encrypt.", note_title);
                                return false;
                            }
                            self.pending_action = Some(PendingCryptoAction::ChangePassword { current_pass: None, new_pass: None });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Change Passphrase: {}", note_title),
                                label: "Enter Current Passphrase:".to_string(),
                                error: None,
                            };
                        }
                    }
                    self.input_buffer.clear();
                    return false;
                }
                KeyCode::Char('e') => {
                    // Contextual toggle encryption for Notebook, Section, or Note
                    match self.focused_pane {
                        Pane::Notebooks => {
                            let nb_name = self.current_notebook_name();
                            let is_enc = self.manager.notebooks.get(self.active_notebook_idx).map(|nb| nb.is_encrypted).unwrap_or(false);
                            if is_enc {
                                self.pending_action = Some(PendingCryptoAction::DecryptCurrentNotebook);
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Decrypt Notebook '{}'", nb_name),
                                    label: "Enter Passphrase to Decrypt:".to_string(),
                                    error: None,
                                };
                            } else {
                                self.pending_action = Some(PendingCryptoAction::EncryptCurrentNotebook { first_pass: None });
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Encrypt Notebook '{}' (Step 1/2)", nb_name),
                                    label: "Enter New Passphrase:".to_string(),
                                    error: None,
                                };
                            }
                        }
                        Pane::Sections => {
                            let sec_name = self.current_section_name();
                            let is_enc = self.manager.notebooks.get(self.active_notebook_idx)
                                .and_then(|nb| nb.sections.get(self.active_section_idx))
                                .map(|sec| sec.is_encrypted)
                                .unwrap_or(false);
                            if is_enc {
                                self.pending_action = Some(PendingCryptoAction::DecryptCurrentSection);
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Decrypt Section '{}'", sec_name),
                                    label: "Enter Passphrase to Decrypt:".to_string(),
                                    error: None,
                                };
                            } else {
                                self.pending_action = Some(PendingCryptoAction::EncryptCurrentSection { first_pass: None });
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Encrypt Section '{}' (Step 1/2)", sec_name),
                                    label: "Enter New Passphrase:".to_string(),
                                    error: None,
                                };
                            }
                        }
                        Pane::Notes | Pane::MainView => {
                            let note_title = self.current_note_title();
                            if self.is_current_note_encrypted() {
                                self.pending_action = Some(PendingCryptoAction::DecryptCurrentNote);
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Decrypt Note '{}'", note_title),
                                    label: "Enter Passphrase to Decrypt:".to_string(),
                                    error: None,
                                };
                            } else {
                                self.pending_action = Some(PendingCryptoAction::EncryptCurrentNote { first_pass: None });
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Encrypt Note '{}' (Step 1/2)", note_title),
                                    label: "Enter New Passphrase:".to_string(),
                                    error: None,
                                };
                            }
                        }
                    }
                    self.input_buffer.clear();
                    return false;
                }
                KeyCode::Char('s') => {
                    if self.view_mode == ViewMode::Editor {
                        self.save_current_editor_content();
                    }
                    return false;
                }
                KeyCode::Char('c') => {
                    if self.view_mode == ViewMode::Editor {
                        self.editor.insert_color_tag("#FF8C00");
                    }
                    return false;
                }
                KeyCode::Char('m') => {
                    if self.view_mode == ViewMode::Editor {
                        self.editor.insert_mermaid_template();
                    }
                    return false;
                }
                _ => {}
            }
        }

        // View Mode specific keys (Editor vs Preview)
        if self.view_mode == ViewMode::Editor {
            match key.code {
                KeyCode::Esc => {
                    if self.editor.is_modified {
                        self.input_mode = InputMode::ConfirmEditorExit;
                    } else {
                        self.view_mode = ViewMode::Preview;
                        self.load_current_note();
                    }
                    return false;
                }
                KeyCode::Up => self.editor.move_up(),
                KeyCode::Down => self.editor.move_down(),
                KeyCode::Left => self.editor.move_left(),
                KeyCode::Right => self.editor.move_right(),
                KeyCode::Home => self.editor.move_home(),
                KeyCode::End => self.editor.move_end(),
                KeyCode::Backspace => self.editor.backspace(),
                KeyCode::Delete => self.editor.delete_char(),
                KeyCode::Enter => self.editor.insert_newline(),
                KeyCode::Char(c) => self.editor.insert_char(c),
                _ => {}
            }
            return false;
        }

        // Normal Preview / Navigation keys
        match key.code {
            KeyCode::Char('q') => return true, // Signal exit
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('w') => {
                self.word_wrap = !self.word_wrap;
                self.status_message = format!("Word Wrap: {}", if self.word_wrap { "ON" } else { "OFF" });
            }
            KeyCode::Char('f') => {
                self.is_fullscreen = !self.is_fullscreen;
                if self.is_fullscreen {
                    self.focused_pane = Pane::MainView;
                    self.status_message = "Entered Fullscreen mode".to_string();
                } else {
                    self.status_message = "Exited Fullscreen mode".to_string();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.focused_pane = match self.focused_pane {
                    Pane::Notes => Pane::MainView,
                    Pane::MainView => Pane::Notes,
                    _ => Pane::Notes,
                };
            }
            KeyCode::Char('b') => {
                self.focused_pane = Pane::Notebooks;
                self.status_message = "Focused: Notebooks".to_string();
            }
            KeyCode::Char('s') => {
                self.focused_pane = Pane::Sections;
                self.status_message = "Focused: Sections".to_string();
            }
            KeyCode::Char('n') => {
                self.focused_pane = Pane::Notes;
                self.status_message = "Focused: Notes".to_string();
            }
            KeyCode::Char('1') => {
                self.focused_pane = Pane::Notebooks;
                self.status_message = "Focused: Notebooks (1)".to_string();
            }
            KeyCode::Char('2') => {
                self.focused_pane = Pane::Sections;
                self.status_message = "Focused: Sections (2)".to_string();
            }
            KeyCode::Char('3') => {
                self.focused_pane = Pane::Notes;
                self.status_message = "Focused: Notes (3)".to_string();
            }
            KeyCode::Char('4') => {
                self.focused_pane = Pane::MainView;
                self.status_message = "Focused: Note Viewer (4)".to_string();
            }
            KeyCode::Left | KeyCode::Char('h') => match self.focused_pane {
                Pane::Notebooks => {
                    if self.active_notebook_idx > 0 {
                        self.active_notebook_idx -= 1;
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    }
                }
                Pane::Sections => {
                    self.focused_pane = Pane::Notebooks;
                }
                Pane::Notes => {
                    self.focused_pane = Pane::Sections;
                }
                Pane::MainView => {
                    self.focused_pane = Pane::Notes;
                }
            },
            KeyCode::Right | KeyCode::Char('l') => match self.focused_pane {
                Pane::Notebooks => {
                    if self.active_notebook_idx + 1 < self.manager.notebooks.len() {
                        self.active_notebook_idx += 1;
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    } else {
                        self.focused_pane = Pane::Sections;
                    }
                }
                Pane::Sections => {
                    self.focused_pane = Pane::Notes;
                }
                Pane::Notes => {
                    self.focused_pane = Pane::MainView;
                }
                Pane::MainView => {}
            },
            KeyCode::F(1) => {
                if !self.manager.notebooks.is_empty() {
                    self.active_notebook_idx = (self.active_notebook_idx + 1) % self.manager.notebooks.len();
                    self.active_section_idx = 0;
                    self.active_note_idx = 0;
                    self.load_current_note();
                }
            }
            KeyCode::Char('e') => {
                if self.is_current_note_encrypted() && self.current_note_content.contains("🔒") {
                    let title = self.current_note_title();
                    self.pending_action = Some(PendingCryptoAction::UnlockNote);
                    self.input_mode = InputMode::PassphrasePrompt {
                        prompt: format!("Unlock Note '{}'", title),
                        label: "Enter Passphrase:".to_string(),
                        error: None,
                    };
                    self.input_buffer.clear();
                } else {
                    self.view_mode = ViewMode::Editor;
                }
            }
            KeyCode::Enter => {
                match self.focused_pane {
                    Pane::Notebooks => {
                        self.focused_pane = Pane::Sections;
                    }
                    Pane::Sections => {
                        self.focused_pane = Pane::Notes;
                    }
                    Pane::Notes | Pane::MainView => {
                        if self.is_current_note_encrypted() && self.current_note_content.contains("🔒") {
                            let title = self.current_note_title();
                            self.pending_action = Some(PendingCryptoAction::UnlockNote);
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Unlock Note '{}'", title),
                                label: "Enter Passphrase:".to_string(),
                                error: None,
                            };
                            self.input_buffer.clear();
                        } else {
                            self.view_mode = ViewMode::Editor;
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                self.launch_external_editor();
            }
            KeyCode::Up | KeyCode::Char('k') => match self.focused_pane {
                Pane::Notebooks => {
                    if self.active_notebook_idx > 0 {
                        self.active_notebook_idx -= 1;
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    }
                }
                Pane::Sections => {
                    if self.active_section_idx > 0 {
                        self.active_section_idx -= 1;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    }
                }
                Pane::Notes => {
                    if self.active_note_idx > 0 {
                        self.active_note_idx -= 1;
                        self.load_current_note();
                    }
                }
                Pane::MainView => {
                    if self.preview_scroll_y > 0 {
                        self.preview_scroll_y -= 1;
                    }
                }
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focused_pane {
                Pane::Notebooks => {
                    if self.active_notebook_idx + 1 < self.manager.notebooks.len() {
                        self.active_notebook_idx += 1;
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    }
                }
                Pane::Sections => {
                    let sec_count = self
                        .manager
                        .notebooks
                        .get(self.active_notebook_idx)
                        .map(|nb| nb.sections.len())
                        .unwrap_or(0);
                    if self.active_section_idx + 1 < sec_count {
                        self.active_section_idx += 1;
                        self.active_note_idx = 0;
                        self.load_current_note();
                    }
                }
                Pane::Notes => {
                    let note_count = self
                        .manager
                        .notebooks
                        .get(self.active_notebook_idx)
                        .and_then(|nb| nb.sections.get(self.active_section_idx))
                        .map(|sec| sec.notes.len())
                        .unwrap_or(0);
                    if self.active_note_idx + 1 < note_count {
                        self.active_note_idx += 1;
                        self.load_current_note();
                    }
                }
                Pane::MainView => {
                    self.preview_scroll_y += 1;
                }
            },
            KeyCode::PageUp => {
                self.preview_scroll_y = self.preview_scroll_y.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.preview_scroll_y += 10;
            }
            _ => {}
        }

        false
    }

    fn handle_passphrase_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.pending_action = None;
            }
            KeyCode::Enter => {
                let passphrase = self.input_buffer.clone();
                self.input_buffer.clear();
                self.process_crypto_action(&passphrase);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_input_dialog(&mut self, key: KeyEvent, item_type: &str) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                let mut name = self.input_buffer.trim().to_string();
                self.input_buffer.clear();
                self.input_mode = InputMode::Normal;

                if name.is_empty() {
                    if item_type == "note" {
                        name = self.config.format_default_note_title();
                    } else if item_type == "section" {
                        name = self.config.format_default_section_title();
                    }
                }

                if !name.is_empty() {
                    match item_type {
                        "notebook" => {
                            let _ = self.manager.create_notebook(&name);
                            self.active_notebook_idx = self.manager.notebooks.len().saturating_sub(1);
                            self.active_section_idx = 0;
                            self.active_note_idx = 0;
                            self.load_current_note();
                        }
                        "section" => {
                            let _ = self.manager.create_section(self.active_notebook_idx, &name);
                            self.load_current_note();
                        }
                        "note" => {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let date_str = crate::versioning::format_timestamp(now);
                            let template_content = self.config.get_template_for(&name, &date_str);

                            if let Ok(_path) = self.manager.create_note(
                                self.active_notebook_idx,
                                self.active_section_idx,
                                &name,
                                false,
                                Some(&template_content),
                            ) {
                                self.load_current_note();
                            }
                        }
                        "rename_notebook" => {
                            let _ = self.manager.rename_notebook(self.active_notebook_idx, &name);
                            self.load_current_note();
                            self.status_message = format!("Renamed notebook to '{}'", name);
                        }
                        "rename_section" => {
                            let _ = self.manager.rename_section(self.active_notebook_idx, self.active_section_idx, &name);
                            self.load_current_note();
                            self.status_message = format!("Renamed section to '{}'", name);
                        }
                        "rename_note" => {
                            let _ = self.manager.rename_note(self.active_notebook_idx, self.active_section_idx, self.active_note_idx, &name);
                            self.load_current_note();
                            self.status_message = format!("Renamed note to '{}'", name);
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_confirm_delete(&mut self, key: KeyEvent, title: &str, item_type: &str) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                match item_type {
                    "notebook" => {
                        let _ = self.manager.delete_notebook(self.active_notebook_idx);
                        self.active_notebook_idx = 0;
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                        self.status_message = format!("Deleted Notebook '{}'", title);
                    }
                    "section" => {
                        let _ = self.manager.delete_section(self.active_notebook_idx, self.active_section_idx);
                        self.active_section_idx = 0;
                        self.active_note_idx = 0;
                        self.load_current_note();
                        self.status_message = format!("Deleted Section '{}'", title);
                    }
                    "note" => {
                        if let Some(note) = self.current_note_file().cloned() {
                            let _ = self.manager.delete_note(&note.path);
                            self.active_note_idx = 0;
                            self.load_current_note();
                            self.status_message = format!("Deleted Note '{}'", title);
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_message = "Deletion cancelled".to_string();
            }
            _ => {}
        }
        false
    }

    fn handle_confirm_editor_exit(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                self.save_current_editor_content();
                self.view_mode = ViewMode::Preview;
                self.input_mode = InputMode::Normal;
                self.status_message = "Saved note changes".to_string();
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.view_mode = ViewMode::Preview;
                self.load_current_note();
                self.input_mode = InputMode::Normal;
                self.status_message = "Discarded unsaved edits".to_string();
            }
            KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') => {
                self.input_mode = InputMode::Normal;
                self.status_message = "Resumed editing".to_string();
            }
            _ => {}
        }
        false
    }

    fn process_crypto_action(&mut self, passphrase: &str) {
        if passphrase.is_empty() {
            let (prompt, label) = match &self.input_mode {
                InputMode::PassphrasePrompt { prompt, label, .. } => (prompt.clone(), label.clone()),
                _ => ("Passphrase Required".to_string(), "Enter Passphrase:".to_string()),
            };
            self.input_mode = InputMode::PassphrasePrompt {
                prompt,
                label,
                error: Some("Passphrase cannot be empty!".to_string()),
            };
            return;
        }

        let action = match self.pending_action.clone() {
            Some(a) => a,
            None => {
                self.input_mode = InputMode::Normal;
                return;
            }
        };

        match action {
            PendingCryptoAction::UnlockNote => {
                if let Some(note) = self.current_note_file() {
                    if let Ok(raw) = self.manager.read_note_raw(&note.path) {
                        match decrypt_note(&raw, passphrase) {
                            Ok(text) => {
                                self.cached_passphrase = Some(passphrase.to_string());
                                self.current_note_content = text.clone();
                                self.editor = Editor::from_string(&text);
                                self.status_message = "Unlocked successfully!".to_string();
                                self.pending_action = None;
                                self.input_mode = InputMode::Normal;
                            }
                            Err(_) => {
                                let title = self.current_note_title();
                                self.pending_action = Some(PendingCryptoAction::UnlockNote);
                                self.input_mode = InputMode::PassphrasePrompt {
                                    prompt: format!("Unlock Note '{}'", title),
                                    label: "Enter Passphrase:".to_string(),
                                    error: Some("Incorrect Passphrase! Please try again.".to_string()),
                                };
                            }
                        }
                    }
                }
            }
            PendingCryptoAction::EncryptCurrentNote { first_pass } => {
                let note_title = self.current_note_title();
                match first_pass {
                    None => {
                        // Step 1 -> Step 2: Confirm
                        self.pending_action = Some(PendingCryptoAction::EncryptCurrentNote {
                            first_pass: Some(passphrase.to_string()),
                        });
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Encrypt Note '{}' (Step 2/2)", note_title),
                            label: "Confirm Passphrase:".to_string(),
                            error: None,
                        };
                    }
                    Some(fp) => {
                        if fp != passphrase {
                            self.pending_action = Some(PendingCryptoAction::EncryptCurrentNote {
                                first_pass: None,
                            });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Encrypt Note '{}' (Step 1/2)", note_title),
                                label: "Enter New Passphrase:".to_string(),
                                error: Some("Passphrases do not match! Please try again.".to_string()),
                            };
                        } else {
                            if let Some(note) = self.current_note_file().cloned() {
                                let content = self.editor.to_string();
                                let enc_path = note.path.with_extension("md.enc");
                                if let Ok(enc_bytes) = encrypt_note(&content, &fp) {
                                    let _ = self.manager.save_note_raw(&enc_path, &enc_bytes);
                                    if note.path != enc_path {
                                        let _ = std::fs::remove_file(&note.path);
                                    }
                                    self.cached_passphrase = Some(fp);
                                    self.manager.reload();
                                    self.load_current_note();
                                    self.status_message = "Note encrypted successfully (Passphrase confirmed)!".to_string();
                                }
                            }
                            self.pending_action = None;
                            self.input_mode = InputMode::Normal;
                        }
                    }
                }
            }
            PendingCryptoAction::DecryptCurrentNote => {
                let note_title = self.current_note_title();
                if let Some(note) = self.current_note_file().cloned() {
                    if let Ok(raw) = self.manager.read_note_raw(&note.path) {
                        if let Ok(plaintext) = decrypt_note(&raw, passphrase) {
                            let dec_path = note.path.with_extension("").with_extension("md");
                            let _ = self.manager.save_note_markdown(&dec_path, &plaintext, None);
                            if note.path != dec_path {
                                let _ = std::fs::remove_file(&note.path);
                            }
                            self.cached_passphrase = Some(passphrase.to_string());
                            self.manager.reload();
                            self.load_current_note();
                            self.status_message = "Note decrypted to plaintext!".to_string();
                            self.pending_action = None;
                            self.input_mode = InputMode::Normal;
                            return;
                        }
                    }
                }
                self.pending_action = Some(PendingCryptoAction::DecryptCurrentNote);
                self.input_mode = InputMode::PassphrasePrompt {
                    prompt: format!("Decrypt Note '{}'", note_title),
                    label: "Enter Passphrase to Decrypt:".to_string(),
                    error: Some("Incorrect Passphrase! Please try again.".to_string()),
                };
            }
            PendingCryptoAction::EncryptCurrentSection { first_pass } => {
                let sec_name = self.current_section_name();
                match first_pass {
                    None => {
                        self.pending_action = Some(PendingCryptoAction::EncryptCurrentSection {
                            first_pass: Some(passphrase.to_string()),
                        });
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Encrypt Section '{}' (Step 2/2)", sec_name),
                            label: "Confirm Passphrase:".to_string(),
                            error: None,
                        };
                    }
                    Some(fp) => {
                        if fp != passphrase {
                            self.pending_action = Some(PendingCryptoAction::EncryptCurrentSection {
                                first_pass: None,
                            });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Encrypt Section '{}' (Step 1/2)", sec_name),
                                label: "Enter New Passphrase:".to_string(),
                                error: Some("Passphrases do not match! Please try again.".to_string()),
                            };
                        } else {
                            match self.manager.encrypt_section(self.active_notebook_idx, self.active_section_idx, &fp) {
                                Ok(count) => {
                                    self.cached_passphrase = Some(fp);
                                    self.load_current_note();
                                    self.status_message = format!("Section '{}' encrypted ({} notes secured)! Passphrase confirmed.", sec_name, count);
                                }
                                Err(e) => {
                                    self.status_message = format!("Failed to encrypt section: {}", e);
                                }
                            }
                            self.pending_action = None;
                            self.input_mode = InputMode::Normal;
                        }
                    }
                }
            }
            PendingCryptoAction::DecryptCurrentSection => {
                let sec_name = self.current_section_name();
                match self.manager.decrypt_section(self.active_notebook_idx, self.active_section_idx, passphrase) {
                    Ok(count) => {
                        self.cached_passphrase = Some(passphrase.to_string());
                        self.load_current_note();
                        self.status_message = format!("Section '{}' decrypted ({} notes decrypted)!", sec_name, count);
                        self.pending_action = None;
                        self.input_mode = InputMode::Normal;
                    }
                    Err(e) => {
                        self.pending_action = Some(PendingCryptoAction::DecryptCurrentSection);
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Decrypt Section '{}'", sec_name),
                            label: "Enter Passphrase to Decrypt:".to_string(),
                            error: Some(format!("Decryption failed: {}", e)),
                        };
                    }
                }
            }
            PendingCryptoAction::EncryptCurrentNotebook { first_pass } => {
                let nb_name = self.current_notebook_name();
                match first_pass {
                    None => {
                        self.pending_action = Some(PendingCryptoAction::EncryptCurrentNotebook {
                            first_pass: Some(passphrase.to_string()),
                        });
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Encrypt Notebook '{}' (Step 2/2)", nb_name),
                            label: "Confirm Passphrase:".to_string(),
                            error: None,
                        };
                    }
                    Some(fp) => {
                        if fp != passphrase {
                            self.pending_action = Some(PendingCryptoAction::EncryptCurrentNotebook {
                                first_pass: None,
                            });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Encrypt Notebook '{}' (Step 1/2)", nb_name),
                                label: "Enter New Passphrase:".to_string(),
                                error: Some("Passphrases do not match! Please try again.".to_string()),
                            };
                        } else {
                            match self.manager.encrypt_notebook(self.active_notebook_idx, &fp) {
                                Ok(count) => {
                                    self.cached_passphrase = Some(fp);
                                    self.load_current_note();
                                    self.status_message = format!("Notebook '{}' encrypted ({} notes secured)! Passphrase confirmed.", nb_name, count);
                                }
                                Err(e) => {
                                    self.status_message = format!("Failed to encrypt notebook: {}", e);
                                }
                            }
                            self.pending_action = None;
                            self.input_mode = InputMode::Normal;
                        }
                    }
                }
            }
            PendingCryptoAction::DecryptCurrentNotebook => {
                let nb_name = self.current_notebook_name();
                match self.manager.decrypt_notebook(self.active_notebook_idx, passphrase) {
                    Ok(count) => {
                        self.cached_passphrase = Some(passphrase.to_string());
                        self.load_current_note();
                        self.status_message = format!("Notebook '{}' decrypted ({} notes decrypted)!", nb_name, count);
                        self.pending_action = None;
                        self.input_mode = InputMode::Normal;
                    }
                    Err(e) => {
                        self.pending_action = Some(PendingCryptoAction::DecryptCurrentNotebook);
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Decrypt Notebook '{}'", nb_name),
                            label: "Enter Passphrase to Decrypt:".to_string(),
                            error: Some(format!("Decryption failed: {}", e)),
                        };
                    }
                }
            }
            PendingCryptoAction::ChangePassword { current_pass, new_pass } => {
                let target_name = match self.focused_pane {
                    Pane::Notebooks => self.current_notebook_name(),
                    Pane::Sections => self.current_section_name(),
                    Pane::Notes | Pane::MainView => self.current_note_title(),
                };

                match (current_pass, new_pass) {
                    (None, None) => {
                        let is_valid = match self.focused_pane {
                            Pane::Notebooks => {
                                self.manager.notebooks.get(self.active_notebook_idx)
                                    .and_then(|nb| nb.sections.iter().find_map(|s| s.notes.iter().find(|n| n.is_encrypted)))
                                    .and_then(|n| self.manager.read_note_raw(&n.path).ok())
                                    .map(|bytes| decrypt_note(&bytes, passphrase).is_ok())
                                    .unwrap_or(true)
                            }
                            Pane::Sections => {
                                self.manager.notebooks.get(self.active_notebook_idx)
                                    .and_then(|nb| nb.sections.get(self.active_section_idx))
                                    .and_then(|s| s.notes.iter().find(|n| n.is_encrypted))
                                    .and_then(|n| self.manager.read_note_raw(&n.path).ok())
                                    .map(|bytes| decrypt_note(&bytes, passphrase).is_ok())
                                    .unwrap_or(true)
                            }
                            Pane::Notes | Pane::MainView => {
                                self.current_note_file()
                                    .and_then(|n| self.manager.read_note_raw(&n.path).ok())
                                    .map(|bytes| decrypt_note(&bytes, passphrase).is_ok())
                                    .unwrap_or(false)
                            }
                        };

                        if !is_valid {
                            self.pending_action = Some(PendingCryptoAction::ChangePassword { current_pass: None, new_pass: None });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Change Passphrase: {}", target_name),
                                label: "Enter Current Passphrase:".to_string(),
                                error: Some("Incorrect current passphrase! Please try again.".to_string()),
                            };
                            return;
                        }

                        self.pending_action = Some(PendingCryptoAction::ChangePassword {
                            current_pass: Some(passphrase.to_string()),
                            new_pass: None,
                        });
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Change Passphrase: {} (Step 2/3)", target_name),
                            label: "Enter NEW Passphrase:".to_string(),
                            error: None,
                        };
                    }
                    (Some(old_p), None) => {
                        self.pending_action = Some(PendingCryptoAction::ChangePassword {
                            current_pass: Some(old_p),
                            new_pass: Some(passphrase.to_string()),
                        });
                        self.input_mode = InputMode::PassphrasePrompt {
                            prompt: format!("Change Passphrase: {} (Step 3/3)", target_name),
                            label: "Confirm NEW Passphrase:".to_string(),
                            error: None,
                        };
                    }
                    (Some(old_p), Some(new_p)) => {
                        if new_p != passphrase {
                            self.pending_action = Some(PendingCryptoAction::ChangePassword {
                                current_pass: Some(old_p),
                                new_pass: None,
                            });
                            self.input_mode = InputMode::PassphrasePrompt {
                                prompt: format!("Change Passphrase: {} (Step 2/3)", target_name),
                                label: "Enter NEW Passphrase:".to_string(),
                                error: Some("New passphrases do not match! Please try again.".to_string()),
                            };
                            return;
                        }

                        match self.focused_pane {
                            Pane::Notebooks => {
                                match self.manager.change_password_notebook(self.active_notebook_idx, &old_p, &new_p) {
                                    Ok(cnt) => {
                                        self.cached_passphrase = Some(new_p);
                                        self.load_current_note();
                                        self.status_message = format!("Passphrase changed for Notebook '{}' ({} notes updated)!", target_name, cnt);
                                    }
                                    Err(e) => {
                                        self.status_message = format!("Failed to change passphrase: {}", e);
                                    }
                                }
                            }
                            Pane::Sections => {
                                match self.manager.change_password_section(self.active_notebook_idx, self.active_section_idx, &old_p, &new_p) {
                                    Ok(cnt) => {
                                        self.cached_passphrase = Some(new_p);
                                        self.load_current_note();
                                        self.status_message = format!("Passphrase changed for Section '{}' ({} notes updated)!", target_name, cnt);
                                    }
                                    Err(e) => {
                                        self.status_message = format!("Failed to change passphrase: {}", e);
                                    }
                                }
                            }
                            Pane::Notes | Pane::MainView => {
                                if let Some(note_path) = self.current_note_file().map(|n| n.path.clone()) {
                                    match self.manager.change_password_note(&note_path, &old_p, &new_p) {
                                        Ok(()) => {
                                            self.cached_passphrase = Some(new_p);
                                            self.load_current_note();
                                            self.status_message = format!("Passphrase changed for Note '{}'!", target_name);
                                        }
                                        Err(e) => {
                                            self.status_message = format!("Failed to change passphrase: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        self.pending_action = None;
                        self.input_mode = InputMode::Normal;
                    }
                    _ => {
                        self.pending_action = None;
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
        }
    }

    pub fn save_current_editor_content(&mut self) {
        if let Some(note) = self.current_note_file().cloned() {
            let markdown = self.editor.to_string();
            let pass = if note.is_encrypted {
                self.cached_passphrase.as_deref().or(Some("notedog"))
            } else {
                None
            };
            if let Ok(()) = self.manager.save_note_markdown(&note.path, &markdown, pass) {
                if let Ok(raw_bytes) = self.manager.read_note_raw(&note.path) {
                    let _ = self.version_manager.create_snapshot(&note.path, &raw_bytes);
                }
                self.editor.is_modified = false;
                self.current_note_content = markdown;
                self.status_message = "Saved note changes & revision snapshot!".to_string();
            }
        }
    }

    fn handle_version_history(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.diff_scroll_y = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_version_idx = self.selected_version_idx.saturating_sub(1);
                self.diff_scroll_y = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.current_versions.is_empty() && self.selected_version_idx + 1 < self.current_versions.len() {
                    self.selected_version_idx += 1;
                    self.diff_scroll_y = 0;
                }
            }
            KeyCode::PageUp => {
                self.diff_scroll_y = self.diff_scroll_y.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.diff_scroll_y += 5;
            }
            KeyCode::Enter => {
                if let Some(version) = self.current_versions.get(self.selected_version_idx) {
                    let v = version.clone();
                    if let Ok(()) = self.version_manager.restore_version(&v) {
                        self.manager.reload();
                        self.load_current_note();
                        self.input_mode = InputMode::Normal;
                        self.status_message = format!("Restored revision from {}", v.formatted_time);
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(version) = self.current_versions.get(self.selected_version_idx) {
                    let v = version.clone();
                    let _ = self.version_manager.delete_version(&v);
                    if let Some(note) = self.current_note_file() {
                        self.current_versions = self.version_manager.list_versions(&note.path).unwrap_or_default();
                    }
                    if self.selected_version_idx >= self.current_versions.len() {
                        self.selected_version_idx = self.current_versions.len().saturating_sub(1);
                    }
                    self.status_message = "Deleted revision snapshot!".to_string();
                }
            }
            KeyCode::Char('c') => {
                self.input_mode = InputMode::VersionCleanup;
                self.selected_preset_idx = 0;
            }
            _ => {}
        }
        false
    }

    fn handle_version_cleanup(&mut self, key: KeyEvent) -> bool {
        let presets_count = 5;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::VersionHistory;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_preset_idx = self.selected_preset_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_preset_idx + 1 < presets_count {
                    self.selected_preset_idx += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(note) = self.current_note_file().cloned() {
                    let deleted = match self.selected_preset_idx {
                        0 => self.version_manager.cleanup_preset_keep_count(&note.path, 5).unwrap_or(0),
                        1 => self.version_manager.cleanup_preset_keep_count(&note.path, 10).unwrap_or(0),
                        2 => self.version_manager.cleanup_preset_keep_count(&note.path, 30).unwrap_or(0),
                        3 => self.version_manager.cleanup_preset_keep_days(&note.path, 30).unwrap_or(0),
                        4 => self.version_manager.purge_all_for_note(&note.path).unwrap_or(0),
                        _ => 0,
                    };
                    self.current_versions = self.version_manager.list_versions(&note.path).unwrap_or_default();
                    self.selected_version_idx = 0;
                    self.input_mode = InputMode::VersionHistory;
                    self.status_message = format!("Cleaned up {} old revisions!", deleted);
                }
            }
            _ => {}
        }
        false
    }

    pub fn launch_external_editor(&mut self) {
        let note_path = match self.current_note_file() {
            Some(n) => n.path.clone(),
            None => return,
        };

        let editor_cmd = if self.config.editor != "builtin" {
            self.config.editor.clone()
        } else {
            env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
        };

        let mut stdout = io::stdout();

        // 1. Temporarily leave terminal raw mode, alternate screen, and restore cursor
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = stdout.flush();

        // 2. Determine shell ($SHELL or /bin/sh) and format safe command
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let path_str = note_path.to_string_lossy();
        let safe_path = path_str.replace('\'', "'\\''");
        let full_cmd = format!("{} '{}'", editor_cmd, safe_path);

        // Spawn editor process with explicit TTY Stdio inheritance across fish/bash/zsh
        let status_res = Command::new(&shell)
            .arg("-c")
            .arg(&full_cmd)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        // Fallback: If shell invocation fails, attempt direct Command::new
        if status_res.is_err() {
            let _ = Command::new(&editor_cmd)
                .arg(&note_path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
        }

        // 3. Restore raw mode, alternate screen, and hide cursor
        let _ = crossterm::terminal::enable_raw_mode();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        );
        let _ = stdout.flush();

        self.needs_clear = true;
        self.load_current_note();
    }
}
