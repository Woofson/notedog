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

    let visible_height = (main_area.height as usize).saturating_sub(2).max(1);
    let visible_width = (main_area.width as usize).saturating_sub(8).max(1);

    let border_color = theme.inactive_border;
    let mod_flag = if editor.is_modified { "*" } else { "" };
    let wrap_badge = if editor.word_wrap { "wrap on" } else { "wrap off" };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.preview_border_style(focused))
        .title_bottom(Line::from(vec![
            Span::styled(
                format!(
                    " ┴─ line {}/{} col {}{} ── {} ── edit ─┘ ",
                    editor.cursor_y + 1,
                    editor.lines.len(),
                    editor.cursor_x + 1,
                    mod_flag,
                    wrap_badge
                ),
                Style::default().fg(border_color),
            ),
        ]));

    if show_title {
        let modified_badge = if editor.is_modified { " [*]" } else { "" };
        let title_str = format!(" {}{}", note_name, modified_badge);
        block = block.title(Span::styled(format!(" {} ", title_str.trim()), theme.preview_title_style(focused)));
    }

    let mut display_lines: Vec<Line> = Vec::new();

    if editor.word_wrap {
        // Line-wrapping mode: wrap long lines onto multiple display lines
        struct VisualRow {
            line_idx: usize,
            chunk_idx: usize,
            chunk_text: String,
            is_cursor_row: bool,
            cursor_col: usize,
        }

        let mut visual_rows: Vec<VisualRow> = Vec::new();
        let mut cursor_vrow_idx = 0;

        for (line_idx, line) in editor.lines.iter().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            if chars.is_empty() {
                let is_cursor = line_idx == editor.cursor_y;
                if is_cursor {
                    cursor_vrow_idx = visual_rows.len();
                }
                visual_rows.push(VisualRow {
                    line_idx,
                    chunk_idx: 0,
                    chunk_text: String::new(),
                    is_cursor_row: is_cursor,
                    cursor_col: 0,
                });
            } else {
                let mut chunk_idx = 0;
                let mut c_start = 0;
                while c_start < chars.len() {
                    let c_end = (c_start + visible_width).min(chars.len());
                    let chunk_text: String = chars[c_start..c_end].iter().collect();
                    let is_cursor = line_idx == editor.cursor_y
                        && (editor.cursor_x >= c_start && (editor.cursor_x < c_end || c_end == chars.len()));
                    
                    if is_cursor {
                        cursor_vrow_idx = visual_rows.len();
                    }

                    visual_rows.push(VisualRow {
                        line_idx,
                        chunk_idx,
                        chunk_text,
                        is_cursor_row: is_cursor,
                        cursor_col: editor.cursor_x.saturating_sub(c_start),
                    });

                    c_start = c_end;
                    chunk_idx += 1;
                }
            }
        }

        let start_vrow = if cursor_vrow_idx < editor.scroll_y {
            cursor_vrow_idx
        } else if cursor_vrow_idx >= editor.scroll_y + visible_height {
            cursor_vrow_idx.saturating_sub(visible_height - 1)
        } else {
            editor.scroll_y
        };
        let end_vrow = (start_vrow + visible_height).min(visual_rows.len());

        for vrow in &visual_rows[start_vrow..end_vrow] {
            let (line_num_str, num_style) = if vrow.chunk_idx == 0 {
                let is_cur = vrow.line_idx == editor.cursor_y;
                let st = if is_cur {
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                (format!("{:>3} │ ", vrow.line_idx + 1), st)
            } else {
                ("    ┆ ".to_string(), Style::default().fg(Color::DarkGray))
            };

            let mut line_spans = vec![Span::styled(line_num_str, num_style)];

            if vrow.is_cursor_row {
                let char_count = vrow.chunk_text.chars().count();
                let cur_x = vrow.cursor_col.min(char_count);

                let before: String = vrow.chunk_text.chars().take(cur_x).collect();
                let cur_char: String = vrow.chunk_text.chars().skip(cur_x).take(1).collect();
                let cur_char = if cur_char.is_empty() { " ".to_string() } else { cur_char };
                let after: String = vrow.chunk_text.chars().skip(cur_x + 1).collect();

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
                line_spans.push(Span::styled(vrow.chunk_text.clone(), theme.fg_style()));
            }

            display_lines.push(Line::from(line_spans));
        }
    } else {
        // Horizontal scrolling mode: preserve single-line alignment and scroll horizontally
        let start_line = if editor.cursor_y < editor.scroll_y {
            editor.cursor_y
        } else if editor.cursor_y >= editor.scroll_y + visible_height {
            editor.cursor_y.saturating_sub(visible_height - 1)
        } else {
            editor.scroll_y
        };
        let end_line = (start_line + visible_height).min(editor.lines.len());

        let scroll_x = if editor.cursor_x < editor.scroll_x {
            editor.cursor_x
        } else if editor.cursor_x >= editor.scroll_x + visible_width {
            editor.cursor_x.saturating_sub(visible_width - 1)
        } else {
            editor.scroll_x
        };

        for i in start_line..end_line {
            let is_current_line = i == editor.cursor_y;
            let line_num_str = format!("{:>3} │ ", i + 1);

            let num_style = if is_current_line {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut line_spans = vec![Span::styled(line_num_str, num_style)];

            let content = &editor.lines[i];
            let char_count = content.chars().count();

            if is_current_line {
                let cur_x = editor.cursor_x.min(char_count);

                let before_str: String = if cur_x > scroll_x {
                    content.chars().skip(scroll_x).take(cur_x - scroll_x).collect()
                } else {
                    String::new()
                };

                let cur_char: String = content.chars().skip(cur_x).take(1).collect();
                let cur_char = if cur_char.is_empty() { " ".to_string() } else { cur_char };

                let used_width = before_str.chars().count() + 1;
                let remaining_width = visible_width.saturating_sub(used_width);
                let after_str: String = content.chars().skip(cur_x + 1).take(remaining_width).collect();

                line_spans.push(Span::styled(before_str, theme.fg_style()));
                line_spans.push(Span::styled(
                    cur_char,
                    Style::default()
                        .bg(theme.primary)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
                line_spans.push(Span::styled(after_str, theme.fg_style()));
            } else {
                let visible_slice: String = content.chars().skip(scroll_x).take(visible_width).collect();
                line_spans.push(Span::styled(visible_slice, theme.fg_style()));
            }

            display_lines.push(Line::from(line_spans));
        }
    }

    let paragraph = Paragraph::new(display_lines).block(block);
    f.render_widget(paragraph, main_area);

    // Toolbar for editor shortcuts
    let toolbar_spans = vec![
        Span::styled(" [Ctrl+S] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Save  ", theme.fg_style()),
        Span::styled(" [Ctrl+W] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Wrap  ", theme.fg_style()),
        Span::styled(" [Ctrl+C] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Color Tag  ", theme.fg_style()),
        Span::styled(" [Ctrl+M] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Mermaid  ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Exit  ", theme.fg_style()),
    ];

    let toolbar_block = Block::default()
        .borders(Borders::NONE)
        .style(theme.bg_style());

    let toolbar = Paragraph::new(Line::from(toolbar_spans)).block(toolbar_block);
    f.render_widget(toolbar, toolbar_area);
}
