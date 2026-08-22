use crate::editor::Editor;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_editor_view(
    f: &mut Frame,
    area: Rect,
    editor: &Editor,
    note_name: &str,
    focused: bool,
    show_title: bool,
    theme: &Theme,
    _icons: &crate::config::IconConfig,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)])
        .split(area);

    let main_area = chunks[0];
    let toolbar_area = chunks[1];

    let visible_height = main_area.height.saturating_sub(2) as usize;

    let border_color = theme.inactive_border;
    let mod_flag = if editor.is_modified { "*" } else { "" };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.preview_border_style(focused))
        .title_bottom(Line::from(vec![
            Span::styled(
                format!(" ┴─ line {}/{}{} ── edit ─┘ ", editor.cursor_y + 1, editor.lines.len(), mod_flag),
                Style::default().fg(border_color),
            ),
        ]));

    if show_title {
        let modified_badge = if editor.is_modified { " [*]" } else { "" };
        let title_str = format!(" {}{}", note_name, modified_badge);
        block = block.title(Span::styled(format!(" {} ", title_str.trim()), theme.preview_title_style(focused)));
    }

    let mut display_lines: Vec<Line> = Vec::new();

    let start_line = editor.scroll_y;
    let end_line = (start_line + visible_height).min(editor.lines.len());

    for i in start_line..end_line {
        let line_num_str = format!("{:>3} │ ", i + 1);
        let is_current_line = i == editor.cursor_y;

        let num_style = if is_current_line {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut line_spans = vec![Span::styled(line_num_str, num_style)];

        let content = &editor.lines[i];
        if is_current_line {
            // Render text with visible cursor indicator
            let char_count = content.chars().count();
            let cur_x = editor.cursor_x.min(char_count);

            let before: String = content.chars().take(cur_x).collect();
            let cur_char: String = content.chars().skip(cur_x).take(1).collect();
            let cur_char = if cur_char.is_empty() { " ".to_string() } else { cur_char };
            let after: String = content.chars().skip(cur_x + 1).collect();

            line_spans.push(Span::styled(before, theme.fg_style()));
            line_spans.push(Span::styled(
                cur_char,
                Style::default()
                    .bg(theme.primary)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::styled(after, theme.fg_style()));
        } else {
            line_spans.push(Span::styled(content.clone(), theme.fg_style()));
        }

        display_lines.push(Line::from(line_spans));
    }

    let paragraph = Paragraph::new(display_lines).block(block);
    f.render_widget(paragraph, main_area);

    // Toolbar for editor shortcuts
    let toolbar_spans = vec![
        Span::styled(" [Ctrl+S] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Save  ", theme.fg_style()),
        Span::styled(" [Ctrl+C] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Insert Color Tag  ", theme.fg_style()),
        Span::styled(" [Ctrl+M] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Insert Mermaid  ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Exit  ", theme.fg_style()),
    ];

    let toolbar_block = Block::default()
        .borders(Borders::NONE)
        .style(theme.bg_style());

    let toolbar = Paragraph::new(Line::from(toolbar_spans)).block(toolbar_block);
    f.render_widget(toolbar, toolbar_area);
}
