use crate::markdown::render_markdown;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    text::Span,
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
    theme: &Theme,
) {
    let rendered_lines = render_markdown(markdown_text, theme);

    let title_prefix = if is_encrypted { " 🔒 " } else { " 📖 " };
    let wrap_badge = if word_wrap { " [Wrap: ON]" } else { " [Wrap: OFF]" };
    let full_title = format!("{}{}{}{} ", title_prefix, title, if is_encrypted { " [Encrypted]" } else { "" }, wrap_badge);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(if focused {
            theme.active_border_style()
        } else {
            theme.border_style()
        })
        .title(if focused {
            Span::styled(format!(" ▶ {} ◀ ", full_title.trim()), theme.active_title_style())
        } else {
            Span::styled(full_title, theme.title_style())
        });

    let mut paragraph = Paragraph::new(rendered_lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    if word_wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);
}
