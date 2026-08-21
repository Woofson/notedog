use crate::crypto::{encrypt_note};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NoteFile {
    pub name: String,
    pub filename: String,
    pub path: PathBuf,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub path: PathBuf,
    pub notes: Vec<NoteFile>,
}

#[derive(Debug, Clone)]
pub struct Notebook {
    pub name: String,
    pub path: PathBuf,
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub struct NoteManager {
    pub root_dir: PathBuf,
    pub notebooks: Vec<Notebook>,
}

impl NoteManager {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut manager = Self {
            root_dir,
            notebooks: Vec::new(),
        };
        manager.ensure_starter_notes();
        manager.reload();
        manager
    }

    pub fn reload(&mut self) {
        self.notebooks.clear();
        if !self.root_dir.exists() {
            let _ = fs::create_dir_all(&self.root_dir);
        }

        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            let mut nb_entries: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir() && !p.file_name().unwrap_or_default().to_string_lossy().starts_with('.'))
                .collect();
            nb_entries.sort();

            for nb_path in nb_entries {
                let nb_name = nb_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let mut sections = Vec::new();
                if let Ok(sec_entries) = fs::read_dir(&nb_path) {
                    let mut sec_paths: Vec<PathBuf> = sec_entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir() && !p.file_name().unwrap_or_default().to_string_lossy().starts_with('.'))
                        .collect();
                    sec_paths.sort();

                    for sec_path in sec_paths {
                        let sec_name = sec_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let mut notes = Vec::new();
                        if let Ok(note_entries) = fs::read_dir(&sec_path) {
                            let mut note_paths: Vec<PathBuf> = note_entries
                                .filter_map(|e| e.ok())
                                .map(|e| e.path())
                                .filter(|p| {
                                    if p.is_file() {
                                        let name = p.file_name().unwrap_or_default().to_string_lossy();
                                        (name.ends_with(".md") || name.ends_with(".md.enc"))
                                            && !name.starts_with('.')
                                    } else {
                                        false
                                    }
                                })
                                .collect();
                            note_paths.sort();

                            for note_path in note_paths {
                                let filename = note_path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                
                                let is_encrypted = filename.ends_with(".md.enc");
                                let name = if is_encrypted {
                                    filename.strip_suffix(".md.enc").unwrap_or(&filename).to_string()
                                } else {
                                    filename.strip_suffix(".md").unwrap_or(&filename).to_string()
                                };

                                notes.push(NoteFile {
                                    name,
                                    filename,
                                    path: note_path,
                                    is_encrypted,
                                });
                            }
                        }

                        sections.push(Section {
                            name: sec_name,
                            path: sec_path,
                            notes,
                        });
                    }
                }

                self.notebooks.push(Notebook {
                    name: nb_name,
                    path: nb_path,
                    sections,
                });
            }
        }
    }

    pub fn create_notebook(&mut self, name: &str) -> io::Result<()> {
        let path = self.root_dir.join(name);
        fs::create_dir_all(&path)?;
        let default_sec = path.join("General");
        fs::create_dir_all(&default_sec)?;
        self.reload();
        Ok(())
    }

    pub fn create_section(&mut self, nb_idx: usize, name: &str) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            let path = nb.path.join(name);
            fs::create_dir_all(&path)?;
            self.reload();
        }
        Ok(())
    }

    pub fn create_note(&mut self, nb_idx: usize, sec_idx: usize, title: &str, is_encrypted: bool) -> io::Result<PathBuf> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            if let Some(sec) = nb.sections.get(sec_idx) {
                let sanitized = title.trim().replace(' ', "_");
                let filename = if is_encrypted {
                    format!("{}.md.enc", sanitized)
                } else {
                    format!("{}.md", sanitized)
                };
                let path = sec.path.join(&filename);
                if !is_encrypted {
                    let default_content = format!("# {}\n\nCreated on Notedog.\n\n- Write your colorful markdown here...\n", title);
                    fs::write(&path, default_content)?;
                } else {
                    let initial = format!("# {} (Encrypted)\n\nThis note is encrypted.\n", title);
                    if let Ok(bytes) = encrypt_note(&initial, "notedog") {
                        fs::write(&path, bytes)?;
                    } else {
                        fs::write(&path, b"")?;
                    }
                }
                self.reload();
                return Ok(path);
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Notebook or Section not found"))
    }

    pub fn read_note_raw(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    pub fn save_note_raw(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        fs::write(path, content)
    }

    pub fn save_note_markdown(&self, path: &Path, markdown_text: &str, password: Option<&str>) -> io::Result<()> {
        if let Some(pass) = password {
            let bytes = encrypt_note(markdown_text, pass)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            fs::write(path, bytes)
        } else {
            fs::write(path, markdown_text)
        }
    }

    pub fn delete_note(&mut self, path: &Path) -> io::Result<()> {
        if path.exists() {
            fs::remove_file(path)?;
            self.reload();
        }
        Ok(())
    }

    pub fn delete_section(&mut self, nb_idx: usize, sec_idx: usize) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            if let Some(sec) = nb.sections.get(sec_idx) {
                if sec.path.exists() {
                    fs::remove_dir_all(&sec.path)?;
                    self.reload();
                }
            }
        }
        Ok(())
    }

    pub fn delete_notebook(&mut self, nb_idx: usize) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            if nb.path.exists() {
                fs::remove_dir_all(&nb.path)?;
                self.reload();
            }
        }
        Ok(())
    }

    pub fn rename_notebook(&mut self, nb_idx: usize, new_name: &str) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            let sanitized = new_name.trim().replace('/', "_");
            if sanitized.is_empty() {
                return Ok(());
            }
            if let Some(parent) = nb.path.parent() {
                let target = parent.join(&sanitized);
                if target != nb.path {
                    fs::rename(&nb.path, &target)?;
                    self.reload();
                }
            }
        }
        Ok(())
    }

    pub fn rename_section(&mut self, nb_idx: usize, sec_idx: usize, new_name: &str) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            if let Some(sec) = nb.sections.get(sec_idx) {
                let sanitized = new_name.trim().replace('/', "_");
                if sanitized.is_empty() {
                    return Ok(());
                }
                if let Some(parent) = sec.path.parent() {
                    let target = parent.join(&sanitized);
                    if target != sec.path {
                        fs::rename(&sec.path, &target)?;
                        self.reload();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn rename_note(&mut self, nb_idx: usize, sec_idx: usize, note_idx: usize, new_name: &str) -> io::Result<()> {
        if let Some(nb) = self.notebooks.get(nb_idx) {
            if let Some(sec) = nb.sections.get(sec_idx) {
                if let Some(note) = sec.notes.get(note_idx) {
                    let sanitized = new_name.trim().replace('/', "_");
                    if sanitized.is_empty() {
                        return Ok(());
                    }
                    let ext = if note.is_encrypted { "md.enc" } else { "md" };
                    let target_filename = format!("{}.{}", sanitized, ext);
                    if let Some(parent) = note.path.parent() {
                        let target_path = parent.join(target_filename);
                        if target_path != note.path {
                            fs::rename(&note.path, &target_path)?;
                            self.reload();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_starter_notes(&self) {
        if self.root_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.root_dir) {
                if entries.count() > 0 {
                    return; // Already initialized
                }
            }
        }

        let personal_sec = self.root_dir.join("Personal").join("General");
        let project_sec = self.root_dir.join("Projects").join("Notedog_Dev");

        let _ = fs::create_dir_all(&personal_sec);
        let _ = fs::create_dir_all(&project_sec);

        let welcome_md = r##"# 🐶 Welcome to Notedog!

Notedog is a modern, vibrant **TUI Notes Application** built in Rust for Linux, Arch & cross-platform enthusiasts.

---

## 🎨 Color-Supported Markdown
Notedog supports rich inline color tags that render directly in the terminal **and** stay fully compatible with standard Markdown readers (like Obsidian, VSCode, or GitHub):

- Use standard HTML spans: <span style="color:#FF8C00">Warm Dark Orange</span> or <span style="color:#FFD700">Gold Accent</span>!
- Or HTML font tags: <font color="#FFA500">Bright Amber Text</font>
- Or quick shorthand: {[#E67E22]Warm Border Color}

### ⚡ Feature Checklist
- [x] OneNote-style hierarchy: `Notebook` > `Section` > `Note`
- [x] Transparent background support (`transparent_background = true`)
- [x] Warm orange and yellow default color palette
- [x] Mermaid & Flowchart rendering right inside the TUI
- [x] Note encryption with ChaCha20-Poly1305 & Argon2id (`.md.enc`)
- [x] Customizable configuration via `~/.config/notedog/notedog.toml`
- [x] Seamless launch of external editor (`$EDITOR`, `nvim`, `nano`)

---

> *"Organization is the key to clarity."*
"##;

        let mermaid_md = r##"# 📊 Mermaid & Flowchart Rendering

Notedog natively parses and renders Mermaid flowcharts inside your terminal!

```mermaid
graph TD
    A[Start Project] --> B{Select Language}
    B -->|Cross-Platform| C[Rust + Ratatui]
    B -->|Quick Prototype| D[Python]
    C --> E[Notedog TUI]
    E --> F[Vibrant Colors]
    E --> G[Encrypted Notes]
    E --> H[Mermaid ASCII Graphs]
```

## 🔄 Left-to-Right Flowchart

```mermaid
graph LR
    Notebook --> Section
    Section --> Note.md
    Note.md --> Preview
    Note.md --> Editor
```
"##;

        let secrets_md = r##"# 🔒 Confidential Project Notes

This note demonstrates **Encrypted Note** capabilities.

- High security encryption using **ChaCha20-Poly1305** cipher.
- Key derivation using **Argon2id**.
- Saved on disk with `.md.enc` extension.

You can encrypt or decrypt any note inside Notedog by pressing `Ctrl+E`!
"##;

        let _ = fs::write(personal_sec.join("01_Welcome.md"), welcome_md);
        let _ = fs::write(personal_sec.join("02_Mermaid_Flowcharts.md"), mermaid_md);
        if let Ok(bytes) = encrypt_note(secrets_md, "notedog") {
            let _ = fs::write(personal_sec.join("03_Secrets.md.enc"), bytes);
        }

        let dev_notes_md = r##"# 🛠️ Arch & Linux Dev Setup

## 📦 System Packages
```bash
sudo pacman -S rustup neovim git
```

## ⚙️ Notedog Config Location
`~/.config/notedog/notedog.toml`

Customize warm orange hues, background transparency, and default `$EDITOR`!
"##;
        let _ = fs::write(project_sec.join("01_Arch_Setup.md"), dev_notes_md);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_note_manager_crud() {
        let temp_dir = std::env::temp_dir().join("notedog_test_dir");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = NoteManager::new(temp_dir.clone());
        assert!(!manager.notebooks.is_empty());

        // Create Notebook & Section
        manager.create_notebook("TestNB").unwrap();
        assert!(manager.notebooks.iter().any(|n| n.name == "TestNB"));

        let nb_idx = manager.notebooks.iter().position(|n| n.name == "TestNB").unwrap();
        manager.create_section(nb_idx, "TestSec").unwrap();

        // Delete Section
        let sec_idx = manager.notebooks[nb_idx].sections.iter().position(|s| s.name == "TestSec").unwrap();
        manager.delete_section(nb_idx, sec_idx).unwrap();
        assert!(!manager.notebooks[nb_idx].sections.iter().any(|s| s.name == "TestSec"));

        // Delete Notebook
        manager.delete_notebook(nb_idx).unwrap();
        assert!(!manager.notebooks.iter().any(|n| n.name == "TestNB"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
