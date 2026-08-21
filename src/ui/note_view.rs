use crate::markdown::render_markdown;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
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
    theme: &Theme,
) {
    let rendered_lines = render_markdown(markdown_text, theme);
    let line_count = rendered_lines.len();

    let title_prefix = if is_encrypted { " 🔒 " } else { " 📖 " };
    let wrap_badge = if word_wrap { "[Wrap: ON]" } else { "[Wrap: OFF]" };
    let full_title = format!("{}{}{} ", title_prefix, title, if is_encrypted { " [Encrypted]" } else { "" });

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if focused {
            theme.active_main_border_style()
        } else {
            theme.border_style()
        })
        .title(if focused {
            Span::styled(format!(" {} ", full_title.trim()), theme.main_title_style())
        } else {
            Span::styled(format!(" {} ", full_title.trim()), theme.title_style())
        })
        .title_bottom(Line::from(vec![
            Span::styled(format!(" ┴─ {} Lines ", line_count), Style::default().fg(theme.border)),
            Span::styled(format!("├── {} ", wrap_badge), Style::default().fg(theme.secondary)),
            Span::styled("├── 👁 Preview ─┘ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]));

    let mut paragraph = Paragraph::new(rendered_lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    if word_wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);
}
