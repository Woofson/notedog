# 🎨 Nerd Fonts Icon Palette for NoteDog

> Ready-to-paste Nerd Fonts icons for your `~/.config/notedog/notedog.toml` configuration file.
> Use these icons if you prefer clean single-color vector icons that respond to custom RGB / Hex tint colors!

---

## 🚀 1. Ready-to-Paste Default `[icons]` Block

Copy and paste this directly into your `~/.config/notedog/notedog.toml`:

```toml
[icons]
notebook       = "󰉋 "
section        = " "
note           = "󰈙 "
encrypted_note = "󰌾 "
preview        = "󰍦 "
editor         = "󰏆 "

# Theme Tint Colors (Now Nerd Font icons will be colored with these hex codes!)
[theme]
notebook_icon_fg          = "#FFCC66" # Ayu Gold
notebook_icon_selected_fg = "#FF7733" # Ayu Coral Orange

section_icon_fg           = "#36A3D9" # Ayu Steel Cyan
section_icon_selected_fg  = "#FFCC66" # Ayu Gold

note_icon_fg              = "#F07178" # Coral Red
note_icon_selected_fg     = "#36A3D9" # Ayu Cyan
```

---

## 📋 2. Ready-to-Paste Regex Icon Rules (`[[icons.rules]]`)

```toml
# 📋 Todo & Task Lists
[[icons.rules]]
pattern = "(?i).*(todo|tasks|tasklist|checklist|to-do).*"
icon = "󰱒 "

# 🛒 Shopping & Grocery Lists
[[icons.rules]]
pattern = "(?i).*(shopping|grocery|groceries|store|buy|buy-list).*"
icon = "󰄗 "

# 💡 Ideas & Brainstorming
[[icons.rules]]
pattern = "(?i).*(idea|ideas|brainstorm|concept).*"
icon = "󰌵 "

# 💼 Work & Projects
[[icons.rules]]
pattern = "(?i).*(work|job|office|project|sprint).*"
icon = "󰲂 "

# 📔 Personal & Journal
[[icons.rules]]
pattern = "(?i).*(personal|journal|diary|daily).*"
icon = "󰠮 "

# 💰 Finance & Budget
[[icons.rules]]
pattern = "(?i).*(finance|budget|money|expense|expenses|bank).*"
icon = "󰄴 "

# 🔒 Secrets & Vault
[[icons.rules]]
pattern = "(?i).*(secret|secrets|passwords|vault|private).*"
icon = "󰌾 "

# 📅 Meetings & Standups
[[icons.rules]]
pattern = "(?i).*(meeting|meetings|call|agenda|standup).*"
icon = "󰃭 "

# 👋 Welcome & Intro
[[icons.rules]]
pattern = "(?i).*(welcome|intro|getting-started|readme).*"
icon = "󰞋 "
```

---

## 🎨 3. Icon Library (Mix & Match)

### 📚 Notebook Icons
| Glyph | Icon Code | Style Description |
| :---: | :--- | :--- |
| `󰉋 ` | `"󰉋 "` | Open Folder/Book (Recommended) |
| `󱓞 ` | `"󱓞 "` | Library Shelf |
| `󰂺 ` | `"󰂺 "` | Closed Journal |
| `󰠮 ` | `"󰠮 "` | Open Book |
| `󰈙 ` | `"󰈙 "` | Stacked Documents |
| `󰓎 ` | `"󰓎 "` | Bookmark Stack |

### 📂 Section Icons
| Glyph | Icon Code | Style Description |
| :---: | :--- | :--- |
| ` ` | `" "` | Open Directory (Recommended) |
| `󰉋 ` | `"󰉋 "` | Closed Directory |
| `󱞁 ` | `"󱞁 "` | Subfolder Branch |
| `󰉌 ` | `"󰉌 "` | Code Folder |
| `󰉍 ` | `"󰉍 "` | Starred Folder |
| `󰉏 ` | `"󰉏 "` | Media Folder |

### 📄 Note & File Icons
| Glyph | Icon Code | Style Description |
| :---: | :--- | :--- |
| `󰈙 ` | `"󰈙 "` | Text Document (Recommended) |
| `󰎞 ` | `"󰎞 "` | Markdown File |
| `󰈔 ` | `"󰈔 "` | Code / Config File |
| `󰏆 ` | `"󰏆 "` | Edit Pencil |
| `󰏗 ` | `"󰏗 "` | Package / Archive |
| `󰍦 ` | `"󰍦 "` | Note Bubble |
| `󰨵 ` | `"󰨵 "` | Terminal Prompt |

### 🔒 Encryption & Security Icons
| Glyph | Icon Code | Style Description |
| :---: | :--- | :--- |
| `󰌾 ` | `"󰌾 "` | Locked Padlock (Recommended) |
| `󰌿 ` | `"󰌿 "` | Key & Lock |
| `󰌽 ` | `"󰌽 "` | Shield Vault |
| `󰌤 ` | `"󰌤 "` | Keyhole |

---

## 💡 How to use:
1. Open `~/.config/notedog/notedog.toml` in your text editor.
2. Replace the `[icons]` block with any of the Nerd Font blocks above.
3. Reload NoteDog — your Nerd Font icons will render cleanly in your chosen theme colors!
