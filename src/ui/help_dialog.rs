use crate::ui::crypto_dialog::centered_rect;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_help_modal(f: &mut Frame, area: Rect, scroll_y: usize, theme: &Theme) {
    let popup_area = centered_rect(75, 75, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " ❓ NOTEDOG HELP & CHEAT SHEET [↑/↓/k/j Scroll] ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let help_lines = vec![
        Line::from(vec![Span::styled("NAVIGATION & PANES", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Tab / Shift+Tab   Move focus between Notebooks, Sections, Notes, and Preview")]),
        Line::from(vec![Span::raw("  ← / → or h / l    Switch active Notebook (when Notebooks tab focused)")]),
        Line::from(vec![Span::raw("  F1                Cycle active Notebook")]),
        Line::from(vec![Span::raw("  ↑ / ↓ / k / j     Navigate Sections / Notes / Scroll note view or help window")]),
        Line::from(vec![Span::raw("  PageUp / PageDown Fast scroll note preview or help window")]),
        Line::from(""),
        Line::from(vec![Span::styled("CREATION & DELETION", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Ctrl+N            Contextual Create (New Note/Section/Notebook depending on focused pane)")]),
        Line::from(vec![Span::raw("  Ctrl+B            Create New Notebook")]),
        Line::from(vec![Span::raw("  Ctrl+K            Create New Section")]),
        Line::from(vec![Span::raw("  r / Ctrl+R        Contextual Rename (Rename focused Notebook, Section, or Note)")]),
        Line::from(vec![Span::raw("  Ctrl+D / d        Contextual Delete (Delete focused Notebook, Section, or Note with confirmation)")]),
        Line::from(""),
        Line::from(vec![Span::styled("EDITING & PREVIEW", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  f / F11 / Ctrl+F  Toggle Fullscreen mode for Editor or Viewer")]),
        Line::from(vec![Span::raw("  w                 Toggle Word Wrap ON/OFF in Note Viewer")]),
        Line::from(vec![Span::raw("  e / Enter         Open Built-in TUI Text Editor")]),
        Line::from(vec![Span::raw("  x                 Launch External Editor ($EDITOR / nvim / nano)")]),
        Line::from(vec![Span::raw("  Ctrl+S            Save note changes in built-in editor")]),
        Line::from(vec![Span::raw("  Ctrl+C            Insert Markdown HTML Color Tag (<span style=\"color:#FF8C00\">)")]),
        Line::from(vec![Span::raw("  Ctrl+M            Insert Mermaid Flowchart Diagram template")]),
        Line::from(""),
        Line::from(vec![Span::styled("ENCRYPTION & DECRYPTION", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Ctrl+E            Encrypt or Decrypt current note with Argon2 + ChaCha20")]),
        Line::from(""),
        Line::from(vec![Span::styled("VERSIONING & REVISIONS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  v / Ctrl+V        Open Revision History modal (View Live Line Diffs, Restore, or Delete snapshots)")]),
        Line::from(vec![Span::raw("  c                 Open Version Cleanup Presets modal (inside Revision History)")]),
        Line::from(vec![Span::raw("  F2 / Ctrl+A       Open About NoteDog page (Author: Bolt J Woofson, GitHub: Woofson/notedog)")]),
        Line::from(""),
        Line::from(vec![Span::styled("COLOR MARKDOWN TAG SYNTAX", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  <span style=\"color:#FF8C00\">Warm Orange</span>")]),
        Line::from(vec![Span::raw("  <font color=\"#FFA500\">Bright Amber</font>")]),
        Line::from(vec![Span::raw("  {[#FFD700]Gold Accent}")]),
        Line::from(""),
        Line::from(vec![Span::styled("CONFIG LOCATION", Style::default().fg(Color::DarkGray))]),
        Line::from(vec![Span::raw("  ~/.config/notedog/notedog.toml")]),
        Line::from(""),
        Line::from(vec![Span::styled("Press [Esc], [q], or [?] to close this help window.", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
    ];

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    f.render_widget(paragraph, popup_area);
}
