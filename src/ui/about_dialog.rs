use crate::theme::Theme;
use crate::ui::crypto_dialog::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_about_modal(f: &mut Frame, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(65, 75, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 🐶 ABOUT NOTEDOG ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let banner = vec![
        Line::from(vec![
            Span::styled("   🐶 NOTEDOG ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("v0.5.0", Style::default().fg(theme.secondary)),
        ]),
        Line::from(Span::styled("   A warm-themed, blazing-fast TUI note application inspired by OneNote & Obsidian", Style::default().fg(theme.foreground))),
        Line::from(""),
    ];

    let banner_para = Paragraph::new(banner);
    f.render_widget(banner_para, inner_chunks[0]);

    let details = vec![
        Line::from(vec![
            Span::styled("AUTHOR & MAINTAINER: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Bolt J Woofson", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("GITHUB USER:         ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Woofson", theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("REPOSITORY:          ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("https://github.com/Woofson/notedog", Style::default().fg(theme.secondary).add_modifier(Modifier::UNDERLINED)),
        ]),
        Line::from(vec![
            Span::styled("LICENSE:             ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("MIT License", theme.fg_style()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("KEY FEATURES:", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  • Hierarchical Notebook > Section > Note structure")]),
        Line::from(vec![Span::raw("  • Universal Color Markdown (<span style=\"color:#...\"> & {[#hex]...})")]),
        Line::from(vec![Span::raw("  • Native ASCII/Unicode Mermaid flowchart renderer (graph TD / LR)")]),
        Line::from(vec![Span::raw("  • ChaCha20-Poly1305 + Argon2id note encryption & decryption")]),
        Line::from(vec![Span::raw("  • Endless file revision history & line-by-line diff viewer")]),
        Line::from(vec![Span::raw("  • Warm orange/yellow palette with transparent terminal support")]),
        Line::from(""),
        Line::from(vec![Span::styled("CONFIG PATH: ", Style::default().fg(Color::DarkGray)), Span::raw("~/.config/notedog/notedog.toml")]),
    ];

    let details_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(" Project Details ", theme.title_style()));

    let details_para = Paragraph::new(details).block(details_block);
    f.render_widget(details_para, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Esc] / [q] / [F2] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Close About Window", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}
