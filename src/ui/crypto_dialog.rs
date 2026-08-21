use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_passphrase_modal(
    f: &mut Frame,
    area: Rect,
    prompt_title: &str,
    input_buffer: &str,
    error_msg: Option<&str>,
    theme: &Theme,
) {
    let popup_area = centered_rect(60, 25, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            format!(" 🔒 {} ", prompt_title),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let label = Paragraph::new("Enter Note Encryption Passphrase:");
    f.render_widget(label, inner_chunks[0]);

    let masked = "*".repeat(input_buffer.len());
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary));

    let input_para = Paragraph::new(format!("{}█", masked)).block(input_block);
    f.render_widget(input_para, inner_chunks[1]);

    if let Some(err) = error_msg {
        let err_para = Paragraph::new(Span::styled(
            format!("⚠️ Error: {}", err),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        f.render_widget(err_para, inner_chunks[2]);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[3]);

    f.render_widget(block, popup_area);
}

pub fn render_input_dialog(
    f: &mut Frame,
    area: Rect,
    dialog_title: &str,
    input_label: &str,
    input_buffer: &str,
    placeholder: Option<&str>,
    theme: &Theme,
) {
    let popup_area = centered_rect(60, 24, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            format!(" 📝 {} ", dialog_title),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let label = Paragraph::new(input_label);
    f.render_widget(label, inner_chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary));

    let input_para = if input_buffer.is_empty() {
        if let Some(ph) = placeholder {
            Paragraph::new(Line::from(vec![
                Span::styled(ph, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                Span::styled("  (Press Enter for default)", Style::default().fg(theme.secondary)),
            ])).block(input_block)
        } else {
            Paragraph::new("█").block(input_block)
        }
    } else {
        Paragraph::new(format!("{}█", input_buffer)).block(input_block)
    };

    f.render_widget(input_para, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_confirm_delete_modal(
    f: &mut Frame,
    area: Rect,
    item_title: &str,
    item_type: &str,
    theme: &Theme,
) {
    let popup_area = centered_rect(60, 25, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.encrypted_tag))
        .title(Span::styled(
            format!(" ⚠️ DELETE {} ", item_type.to_uppercase()),
            Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let warning_text = format!(
        "Are you sure you want to delete {} '{}'?",
        item_type, item_title
    );
    let label = Paragraph::new(vec![
        Line::from(Span::styled(warning_text, Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(
            "This operation cannot be undone and will delete files from disk.",
            Style::default().fg(Color::Red),
        )),
    ]);
    f.render_widget(label, inner_chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [y / Enter] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm Delete   ", theme.fg_style()),
        Span::styled(" [n / Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}
