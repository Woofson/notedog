use crate::diff::{compute_line_diff, DiffLine};
use crate::theme::Theme;
use crate::ui::crypto_dialog::centered_rect;
use crate::versioning::VersionInfo;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::fs;

pub fn render_version_history_modal(
    f: &mut Frame,
    area: Rect,
    versions: &[VersionInfo],
    selected_idx: usize,
    diff_scroll_y: usize,
    current_note_text: &str,
    note_name: &str,
    theme: &Theme,
) {
    let popup_area = centered_rect(85, 85, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            format!(" 🕒 REVISION HISTORY & LIVE DIFF: {} (Total: {}) ", note_name, versions.len()),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(50),
            Constraint::Length(2),
        ])
        .split(popup_area);

    let list_area = main_layout[0];
    let diff_area = main_layout[1];
    let footer_area = main_layout[2];

    let items: Vec<ListItem> = if versions.is_empty() {
        vec![ListItem::new(" (No historical revisions saved yet. Revisions are created automatically when saving.)")]
    } else {
        versions
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let is_selected = i == selected_idx;
                let rev_num = versions.len() - i;

                let style = if is_selected {
                    theme.highlight_style()
                } else {
                    theme.fg_style()
                };

                let prefix = if is_selected { "▶ " } else { "  " };
                let enc_str = if v.is_encrypted { " [🔒 Encrypted Payload]" } else { "" };
                let size_str = format!("{} B", v.size_bytes);

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme.accent)),
                    Span::styled(format!("Rev #{:<3} ", rev_num), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("│ {} ", v.formatted_time), style),
                    Span::styled(format!("│ {:>8} ", size_str), Style::default().fg(theme.secondary)),
                    Span::styled(enc_str, Style::default().fg(theme.encrypted_tag)),
                ]))
            })
            .collect()
    };

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(" Saved Snapshots ", theme.title_style()));

    let list = List::new(items).block(list_block);
    f.render_widget(list, list_area);

    // Compute live diff preview for selected version
    let diff_lines = if let Some(version) = versions.get(selected_idx) {
        if version.is_encrypted {
            vec![Line::from(Span::styled(
                "🔒 Selected revision is encrypted. Differences cannot be shown without unlocking.",
                Style::default().fg(theme.encrypted_tag),
            ))]
        } else if let Ok(rev_bytes) = fs::read(&version.version_path) {
            let rev_text = String::from_utf8_lossy(&rev_bytes);
            let diffs = compute_line_diff(current_note_text, &rev_text);

            if diffs.is_empty() {
                vec![Line::from(Span::styled(
                    " (No line differences - content is identical)",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                diffs
                    .into_iter()
                    .map(|d| match d {
                        DiffLine::Unchanged(line) => Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(Color::DarkGray),
                        )),
                        DiffLine::Added(line) => Line::from(Span::styled(
                            format!("+ {}", line),
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        )),
                        DiffLine::Deleted(line) => Line::from(Span::styled(
                            format!("- {}", line),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )),
                    })
                    .collect()
            }
        } else {
            vec![Line::from(Span::styled(
                "⚠️ Failed to read revision snapshot payload",
                Style::default().fg(Color::Red),
            ))]
        }
    } else {
        vec![Line::from(Span::styled(
            "Select a revision snapshot above to view line differences.",
            Style::default().fg(theme.foreground),
        ))]
    };

    let diff_title = if let Some(v) = versions.get(selected_idx) {
        format!(" 🔍 Diff: Current Note vs Rev {} ({}) [PgUp/PgDn Scroll Diff] ", versions.len() - selected_idx, v.formatted_time)
    } else {
        " 🔍 Line Diff Preview ".to_string()
    };

    let diff_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary))
        .title(Span::styled(diff_title, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));

    let diff_paragraph = Paragraph::new(diff_lines)
        .block(diff_block)
        .scroll((diff_scroll_y as u16, 0));

    f.render_widget(diff_paragraph, diff_area);

    let footer_spans = vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Restore Version   ", theme.fg_style()),
        Span::styled(" [d] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Delete Selected   ", theme.fg_style()),
        Span::styled(" [c] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Cleanup Presets   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Close", theme.fg_style()),
    ];

    let footer = Paragraph::new(Line::from(footer_spans));
    f.render_widget(footer, footer_area);

    f.render_widget(block, popup_area);
}

pub fn render_version_cleanup_modal(
    f: &mut Frame,
    area: Rect,
    presets: &[&str],
    selected_preset_idx: usize,
    note_name: &str,
    theme: &Theme,
) {
    let popup_area = centered_rect(65, 50, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.secondary))
        .title(Span::styled(
            format!(" 🧹 CLEANUP HISTORY PRESETS: {} ", note_name),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let header_text = Paragraph::new(Span::styled(
        "Select a cleanup preset to purge old historical revisions:",
        Style::default().fg(theme.foreground),
    ));
    f.render_widget(header_text, inner_chunks[0]);

    let items: Vec<ListItem> = presets
        .iter()
        .enumerate()
        .map(|(i, &preset)| {
            let is_selected = i == selected_preset_idx;
            let style = if is_selected {
                theme.highlight_style()
            } else {
                theme.fg_style()
            };
            let prefix = if is_selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                Span::styled(format!("{}. {}", i + 1, preset), style),
            ]))
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let list = List::new(items).block(list_block);
    f.render_widget(list, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Execute Preset   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}
