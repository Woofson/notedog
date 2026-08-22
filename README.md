# 🐶 Notedog (v0.5.2)

> A vibrant, cross-platform TUI Notes application in Rust inspired by OneNote and Obsidian. Built by **Bolt J Woofson** (`https://github.com/Woofson/notedog`). Features Smart Note Templates with interactive tickboxes, Superfile & Ayu Dark inspired UI layout, customizable regex icon rules, resizable layout dimensions, maximum per-item color & background customizability, endless file revision history, universal Markdown colors, native Mermaid flowcharts, and ChaCha20-Poly1305 + Argon2id note encryption.

---

## 📸 Overview

**Notedog** is built for Linux (Arch main target) and cross-platform terminal enthusiasts who want an organized, keyboard-driven note-taking system. Notes are saved as plain Markdown files (`.md`) or encrypted binary notes (`.md.enc`), maintaining 100% compatibility with external Markdown editors like Obsidian, VS Code, and GitHub.

---

## ✨ Features

- **📝 Smart Note Templates with Tickboxes**:
  - Auto-populates newly created notes based on title keyword matching (e.g. `Todo` with interactive task tickboxes, `Shopping` / `Grocery` with categorized produce/dairy tickbox lists, `Meeting` / `Agenda` with action items, `Ideas`, `Work`, `Finance`).
  - Supports custom template rules in `notedog.toml` using `{{title}}` and `{{date}}` variables.
- **🎨 Maximum Customizability & Per-Pane Item Styling**:
  - Independent selected and unselected background fills (`notebook_item_selected_bg`, `section_item_selected_bg`, `note_item_selected_bg`).
  - Independent text colors, icon colors, and font weights (`"bold"`, `"dim"`, `"italic"`, `"underlined"`, `"crossed_out"`).
  - Human-understandable color naming (`active_border`, `inactive_border`, `sidebar_title`, `active_sidebar_border`, `foreground`, `background`).
- **📚 Superfile-Style 3-Tier Navigation**: Organize notes into `Notebook` > `Section` (Subject/Project) > `Note.md` with glowing rounded borders (`BorderType::Rounded`) and bottom status badges. Full vertical navigation (`↑`/`↓` and `k`/`j`) for all 3 list boxes.
- **📐 Resizable Layout Dimensions**: Set custom sidebar width and box heights as percentages (`"26%"`, `"34%"`) or fixed terminal rows/columns (`"30"`, `"12"`) in `notedog.toml`.
- **🎨 Custom Icons, Nerd Fonts & Regex Rules**:
  - Full support for both color Emojis (`📚`, `📂`, `📄`) and **Nerd Fonts vector glyphs** (`󰉋 `, ` `, `󰈙 `, `󰌾 `).
  - Built-in preset rules for Todo (`󰱒 `), Shopping (`󰄗 `), Ideas (`󰌵 `), Work (`󰲂 `), Finance (`󰄴 `), Meetings (`󰃭 `), and Secrets (`󰌾 `).
  - Includes a dedicated [Nerd Fonts copy-paste cheat sheet](file:///home/bolt/projects/arfnotes/nerd.md) (`nerd.md`).
- **🎨 Universal Markdown Color Support**:
  - Uses standard HTML spans (`<span style="color:#FF8C00">text</span>`) and font tags (`<font color="gold">text</font>`).
  - Render colors inside the TUI **and** stay fully readable by Obsidian, VS Code, and GitHub Markdown previews.
  - Includes quick color insertion shortcut (`Ctrl+C`) in the editor.
- **📊 Native Mermaid Flowchart Engine**:
  - Automatically parses ` ```mermaid ` code blocks (`graph TD`, `graph LR`).
  - Renders flowchart nodes with Unicode box-drawing shapes (`┌──┐`, `╭──╮`, `◇`) and directional arrows (`▼`, `──►`) directly in the terminal.
- **🔒 Secure Note Encryption**:
  - Encrypt/decrypt individual notes using **ChaCha20-Poly1305** symmetric AEAD cipher with **Argon2id** key derivation.
  - Encrypted notes are saved with `.md.enc` extension and indicated with a 🔒 lock badge in the file browser.
- **📜 Endless Revision History & Live Diffs**:
  - Automated timestamped snapshots on every note edit.
  - Interactive version browser modal (`v` or `Ctrl+V`) with live line-by-line diffs (`+`/`-`), single-click restoration, and cleanup presets.
- **✏️ Flexible Editing & Cross-Shell Support**:
  - Built-in TUI text editor with live syntax colors, line numbers, cursor highlighting, and shortcut toolbars.
  - Launch external editors (`$EDITOR`, `nvim`, `nano`, `micro`) with `x` key across `fish`, `bash`, and `zsh` with automatic screen flush and redraw.

---

## ⌨️ Keybindings

| Keybinding | Action |
| :--- | :--- |
| `Tab` / `Shift+Tab` | Cycle focus between **Notebooks**, **Sections**, **Notes**, and **Main View** |
| `↑` / `↓` / `k` / `j` | Navigate list items vertically in **Notebooks**, **Sections**, or **Notes** list boxes, or scroll note preview |
| `←` / `→` or `h` / `l` | Switch active **Notebook** or navigate panes |
| `F1` | Cycle active **Notebook** tab |
| `PageUp` / `PageDown` | Fast scroll note preview |
| `f` / `F11` / `Ctrl+F` | **Toggle Fullscreen Mode** for Editor or Viewer |
| `w` | **Toggle Word Wrap ON/OFF** in Note Viewer |
| `e` / `Enter` | Open built-in TUI text editor (or unlock encrypted note) |
| `x` | Launch external editor (`$EDITOR` / `nvim` / `nano`) |
| `Ctrl+S` | Save current note (in built-in editor) |
| `Ctrl+C` | Insert HTML Color tag (`<span style="color:#FF8C00">`) |
| `Ctrl+M` | Insert Mermaid flowchart template |
| `Ctrl+N` | **Contextual Create**: Create Notebook, Section, or Note depending on focused pane (applies smart templates with tickboxes) |
| `Ctrl+B` | Create a new **Notebook** |
| `Ctrl+K` | Create a new **Section** |
| `r` / `Ctrl+R` | **Contextual Rename**: Rename focused Notebook, Section, or Note on disk |
| `Ctrl+D` / `d` | **Contextual Delete**: Safely delete focused Notebook, Section, or Note with confirmation dialog |
| `v` / `Ctrl+V` | **Revision History Modal**: Browse endless file version snapshots, view live line diffs (`+`/`-`), restore past revisions, or open cleanup presets |
| `Ctrl+E` | Encrypt or Decrypt current note (prompts for passphrase) |
| `F2` / `Ctrl+A` | Open **About NoteDog** page (Author: **Bolt J Woofson**, Repository: `Woofson/notedog`) |
| `?` | Toggle interactive Help & Shortcut cheat sheet modal |
| `q` | Quit Notedog |

---

## 🚀 CLI Usage & System Initialization

```bash
# Run Notedog normally
notedog

# Verify/initialize clean starter notes (SAFEGUARD: Preserves existing notes!)
notedog --clean

# Print CLI help and storage paths
notedog --help

# Print version and author information
notedog --version
```

---

## ⚙️ Configuration Example (`~/.config/notedog/notedog.toml`)

```toml
note_folder = "~/Notes"
editor = "builtin"             # "builtin", "nvim", "nano", "micro"
secrets_file = "~/.config/notedog/secrets.toml"
transparent_background = true
show_help_bar = true
word_wrap = true
default_notebook = "Personal"

# 📐 RESIZABLE LAYOUT DIMENSIONS
[layout]
sidebar_width    = "26%"      # Percentage ("26%") or fixed columns ("30")
notebooks_height = "26%"      # Percentage ("26%") or fixed rows ("8")
sections_height  = "34%"      # Percentage ("34%") or fixed rows ("10")
notes_height     = "40%"      # Percentage ("40%") or fixed rows ("12")

# 🏷️ CUSTOMIZABLE PANE TITLES (SIDEBAR HEADERS & MAIN VIEW)
[titles]
notebooks       = "NOTEBOOKS" # Title text for Notebooks pane
sections        = "SECTIONS"  # Title text for Sections pane
notes           = "NOTES"     # Title text for Notes pane
show_main_title = false       # Show/hide redundant note title header on the main viewer pane

# 🎨 CUSTOMIZABLE ICONS & REGEX PATTERN RULES
# Rules support optional `target = "notebook"` | `"section"` | `"note"` scoping
[icons]
notebook       = "📚 "
section        = "📂 "
note           = "📄 "
encrypted_note = "🔒 "
preview        = "📖 "
editor         = "✏️ "

[[icons.rules]]
pattern = "(?i).*(todo|tasks|tasklist|checklist|to-do).*"
icon = "✅ "

# General rule for groceries:
[[icons.rules]]
pattern = "(?i).*(shopping|grocery|groceries|store|buy|buy-list).*"
icon = "🛒 "

# Specific icon only when a Note is named groceries:
# [[icons.rules]]
# pattern = "(?i).*groceries.*"
# target = "note"
# icon = "🥦 "

# 📝 CUSTOM NOTE TEMPLATES (OPTIONAL)
[[templates]]
pattern = "(?i).*(sprint|retro).*"
template = """# 🚀 {{title}}

**Date**: {{date}}

## 🟢 What Went Well
- 

## 🔴 What Needs Improvement
- 

## ⚡ Action Items
- [ ] 
"""

# 🎨 THEME CONFIGURATION
# Theme files are stored in ~/.config/notedog/themes/<theme>.toml
# Built-in themes include: "notedog", "nord", "catppuccin-mocha", "catppuccin-latte",
# "dracula", "gruvbox", "tokyo-night", "ayu-dark", "solarized-dark", "monokai"
theme = "notedog"
```

### 🎨 Custom Themes (`~/.config/notedog/themes/`)
Create any `.toml` file in `~/.config/notedog/themes/` (e.g. `my-theme.toml`), and set `theme = "my-theme"` in `notedog.toml`:

```toml
active_border         = "#FFCC66" # Active main window border color
inactive_border       = "#242936" # Inactive window border color
sidebar_title         = "#36A3D9" # Sidebar title headers color
active_sidebar_border = "#FF7733" # Active sidebar block border
foreground            = "#B3B1AD" # Body text foreground color
background            = "none"    # Canvas background ("none" for transparent, or "#0F1419")
highlight_bg          = "#1F2430" # Selected list item background fill
highlight_fg          = "#36A3D9" # Selected list item text color
encrypted_tag         = "#F07178" # Encrypted note lock badge color

# Per-pane item styling & selection background overrides (optional):
note_item_bg                  = "none"
note_item_fg                  = "#B3B1AD"
note_item_weight              = "normal"
note_item_selected_bg         = "#1F2430"
note_item_selected_fg         = "#36A3D9"
note_item_selected_weight     = "bold"
note_icon_fg                  = "#36A3D9"
note_icon_selected_fg         = "#36A3D9"
```

---

## 📦 Installation & Running

### Arch Linux (AUR)
```bash
# Using yay
yay -S notedog

# Using paru
paru -S notedog
```

### Building from Source
```bash
# Clone repository
git clone https://github.com/Woofson/notedog.git
cd notedog

# Run unit tests
cargo test

# Build optimized release binary
cargo build --release
./target/release/notedog
```

---

## 📄 License

MIT License. Author: **Bolt J Woofson** (`https://github.com/Woofson/notedog`).
