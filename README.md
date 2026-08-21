# 🐶 Notedog (v0.4.0)

> A vibrant, cross-platform TUI Notes application in Rust inspired by OneNote and Obsidian. Built by **Bolt J Woofson** (`https://github.com/Woofson/notedog`). Features Superfile & Ayu Dark inspired UI layout, customizable regex icon rules, resizable layout dimensions, human-understandable theme colors, endless file revision history, universal Markdown colors, native Mermaid flowcharts, and ChaCha20-Poly1305 + Argon2id note encryption.

---

## 📸 Overview

**Notedog** is built for Linux (Arch main target) and cross-platform terminal enthusiasts who want an organized, keyboard-driven note-taking system. Notes are saved as plain Markdown files (`.md`) or encrypted binary notes (`.md.enc`), maintaining 100% compatibility with external Markdown editors like Obsidian, VS Code, and GitHub.

---

## ✨ Features

- **📚 Superfile-Style 3-Tier Navigation**: Organize notes into `Notebook` > `Section` (Subject/Project) > `Note.md` with glowing rounded borders (`BorderType::Rounded`) and bottom status badges.
- **📐 Resizable Layout Dimensions**: Set custom sidebar width and box heights as percentages (`"26%"`, `"34%"`) or fixed terminal rows/columns (`"30"`, `"12"`) in `notedog.toml`.
- **🎨 Regex-Based Custom Icons**: Assign custom icons to Notebooks, Sections, or Notes using Regex pattern rules (`[[icons.rules]]`).
- **🖌️ Human-Understandable Theme Colors & Item Styling**:
  - Configure colors using human-understandable names (`active_border`, `inactive_border`, `sidebar_title`, `active_sidebar_border`, `foreground`, `background`).
  - Set individual foreground colors and font weights (`"normal"`, `"bold"`, `"dim"`, `"italic"`) for Notebook, Section, and Note items.
  - Per-pane title and border color overrides (`notebook_border_active`, `section_title_active`, `preview_border_active`, etc.).
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
| `Ctrl+N` | **Contextual Create**: Create Notebook, Section, or Note depending on focused pane (with preview of auto-generated titles) |
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

# 🎨 CUSTOMIZABLE ICONS & REGEX PATTERN RULES
[icons]
notebook       = "📚 "
section        = "📂 "
note           = "📄 "
encrypted_note = "🔒 "
preview        = "📖 "
editor         = "✏️ "

[[icons.rules]]
pattern = ".*Welcome.*"
icon = "👋 "

[[icons.rules]]
pattern = "^Work.*"
icon = "💼 "

# 🎨 HUMAN-UNDERSTANDABLE THEME COLORS & ITEM TEXT STYLING
[theme]
active_border         = "#FFCC66" # Active main window border color (Ayu Gold)
inactive_border       = "#242936" # Inactive window border color (Ayu Charcoal Slate)
sidebar_title         = "#36A3D9" # Sidebar title headers color (Ayu Cyan)
active_sidebar_border = "#FF7733" # Active sidebar block border (Ayu Coral Orange)
foreground            = "#B3B1AD" # Body text foreground color (Soft Off-White)
background            = "none"    # Canvas background ("none" for transparent, or "#0F1419")
highlight_bg          = "#1F2430" # Selected list item background fill (Dark Slate)
highlight_fg          = "#36A3D9" # Selected list item text color (Ayu Cyan)
encrypted_tag         = "#F07178" # Encrypted note tag & lock badge color (Coral Red)

# Item font color and weight overrides:
notebook_item_fg              = "#B3B1AD"
notebook_item_weight          = "normal"   # "normal", "bold", "dim", "italic"
notebook_item_selected_fg     = "#36A3D9"
notebook_item_selected_weight = "bold"
```

---

## 📦 Building & Running

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
