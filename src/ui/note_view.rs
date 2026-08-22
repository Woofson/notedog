use std::path::Path;
use crate::markdown::render_markdown;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_note_preview(
    f: &mut Frame,
    area: Rect,
    title: &str,
    markdown_text: &str,
    scroll_y: usize,
    focused: bool,
    is_encrypted: bool,
    word_wrap: bool,
    show_title: bool,
    base_dir: Option<&Path>,
    theme: &Theme,
    icons: &crate::config::IconConfig,
) {
    let rendered_lines = render_markdown(markdown_text, theme, base_dir);
    let line_count = rendered_lines.len();

    let wrap_label = if word_wrap { "wrap on" } else { "wrap off" };
    let border_color = theme.inactive_border;

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.preview_border_style(focused))
        .title_bottom(Line::from(vec![
            Span::styled(
                format!(" ┴─ {} lines ── {} ── view ─┘ ", line_count, wrap_label),
                Style::default().fg(border_color),
            ),
        ]));

    if show_title {
        let title_prefix = if is_encrypted { &icons.encrypted_note } else { "" };
        let full_title = format!("{}{}{} ", title_prefix, title, if is_encrypted { " [Encrypted]" } else { "" });
        block = block.title(Span::styled(format!(" {} ", full_title.trim()), theme.preview_title_style(focused)));
    }

    let mut paragraph = Paragraph::new(rendered_lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    if word_wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);
}
