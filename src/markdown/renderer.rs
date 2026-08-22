use crate::markdown::color_parser::parse_color_tags;
use crate::mermaid::{parse_mermaid, render_mermaid_to_lines};
use crate::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render_markdown(markdown_text: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;
    let mut is_mermaid_block = false;
    let mut mermaid_buffer = String::new();
    let mut code_lang;

    for line in markdown_text.lines() {
        let trimmed = line.trim();

        // Check for code block fences
        if trimmed.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                code_lang = trimmed.trim_start_matches('`').trim().to_string();
                if code_lang.to_lowercase() == "mermaid" {
                    is_mermaid_block = true;
                    mermaid_buffer.clear();
                } else {
                    is_mermaid_block = false;
                    let fence_label = if code_lang.is_empty() {
                        " Code ".to_string()
                    } else {
                        format!(" Code: {} ", code_lang)
                    };
                    lines.push(Line::from(vec![
                        Span::styled("┌──", Style::default().fg(theme.border)),
                        Span::styled(
                            fence_label,
                            Style::default()
                                .fg(theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("────────────────────────────────", Style::default().fg(theme.border)),
                    ]));
                }
            } else {
                // Closing fence
                in_code_block = false;
                if is_mermaid_block {
                    if let Some(diagram) = parse_mermaid(&mermaid_buffer) {
                        let m_lines = render_mermaid_to_lines(&diagram, theme);
                        lines.extend(m_lines);
                    } else {
                        lines.push(Line::from(vec![Span::styled(
                            " [Mermaid Diagram Parsing Failed - Invalid Syntax] ",
                            Style::default().fg(Color::Red),
                        )]));
                        for mline in mermaid_buffer.lines() {
                            lines.push(Line::from(vec![Span::styled(
                                format!("  {}", mline),
                                Style::default().fg(Color::DarkGray),
                            )]));
                        }
                    }
                    is_mermaid_block = false;
                    mermaid_buffer.clear();
                } else {
                    lines.push(Line::from(vec![Span::styled(
                        "└────────────────────────────────────────",
                        Style::default().fg(theme.border),
                    )]));
                }
            }
            continue;
        }

        if in_code_block {
            if is_mermaid_block {
                mermaid_buffer.push_str(line);
                mermaid_buffer.push('\n');
            } else {
                lines.push(Line::from(vec![
                    Span::styled("│ ", Style::default().fg(theme.border)),
                    Span::styled(
                        format!("{}", line),
                        Style::default().fg(theme.secondary).bg(theme.code_bg),
                    ),
                ]));
            }
            continue;
        }

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            lines.push(Line::from(vec![Span::styled(
                "──────────────────────────────────────────────",
                Style::default().fg(theme.border),
            )]));
            continue;
        }

        // Headers
        if line.starts_with("# ") {
            let text = &line[2..];
            lines.push(Line::from(vec![
                Span::styled("📌 ", Style::default().fg(theme.primary)),
                Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.header_1)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
            ]));
            lines.push(Line::from(""));
            continue;
        } else if line.starts_with("## ") {
            let text = &line[3..];
            lines.push(Line::from(vec![
                Span::styled("🔸 ", Style::default().fg(theme.secondary)),
                Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.header_2)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
            continue;
        } else if line.starts_with("### ") {
            let text = &line[4..];
            lines.push(Line::from(vec![
                Span::styled("🔹 ", Style::default().fg(theme.accent)),
                Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.header_3)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            continue;
        }

        // Blockquotes
        if line.starts_with("> ") {
            let text = &line[2..];
            let spans = build_inline_spans(text, theme, Some(theme.secondary), true, false);
            let mut line_spans = vec![Span::styled(" ▍ ", Style::default().fg(theme.primary))];
            line_spans.extend(spans);
            lines.push(Line::from(line_spans));
            continue;
        }

        // Lists & Checkboxes
        if trimmed.starts_with("- [ ] ") {
            let text = &trimmed[6..];
            let spans = build_inline_spans(text, theme, None, false, false);
            let mut line_spans = vec![Span::styled("  [ ] ", Style::default().fg(theme.secondary))];
            line_spans.extend(spans);
            lines.push(Line::from(line_spans));
            continue;
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            let text = &trimmed[6..];
            let spans = build_inline_spans(text, theme, Some(Color::Green), false, false);
            let mut line_spans = vec![Span::styled("  [✔] ", Style::default().fg(Color::Green))];
            line_spans.extend(spans);
            lines.push(Line::from(line_spans));
            continue;
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            let spans = build_inline_spans(text, theme, None, false, false);
            let mut line_spans = vec![Span::styled("  • ", Style::default().fg(theme.primary))];
            line_spans.extend(spans);
            lines.push(Line::from(line_spans));
            continue;
        }

        // Normal paragraph with HTML & Custom inline color parsing
        let spans = build_inline_spans(line, theme, None, false, false);
        lines.push(Line::from(spans));
    }

    lines
}

fn build_inline_spans(
    text: &str,
    theme: &Theme,
    default_fg: Option<Color>,
    is_italic: bool,
    is_bold: bool,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let color_spans = parse_color_tags(text);

    for cs in color_spans {
        let fg = cs.fg_color.or(default_fg).unwrap_or(theme.foreground);
        let mut style = Style::default().fg(fg);
        if is_italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if is_bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        // Parse standard markdown bold ** / inline code ` inside text
        parse_basic_markdown_tokens(&cs.text, style, theme, &mut spans);
    }

    spans
}

fn parse_basic_markdown_tokens(
    text: &str,
    base_style: Style,
    theme: &Theme,
    out: &mut Vec<Span<'static>>,
) {
    if !text.contains('*') && !text.contains('`') {
        out.push(Span::styled(text.to_string(), base_style));
        return;
    }

    // Split by backticks for inline code first
    let parts: Vec<&str> = text.split('`').collect();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 1 {
            // Inline code segment
            out.push(Span::styled(
                format!(" {} ", part),
                Style::default().fg(theme.accent).bg(theme.code_bg),
            ));
        } else {
            // Check for bold **text**
            if part.contains("**") {
                let bold_parts: Vec<&str> = part.split("**").collect();
                for (j, bpart) in bold_parts.iter().enumerate() {
                    if j % 2 == 1 {
                        out.push(Span::styled(
                            bpart.to_string(),
                            base_style.add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        out.push(Span::styled(bpart.to_string(), base_style));
                    }
                }
            } else {
                out.push(Span::styled(part.to_string(), base_style));
            }
        }
    }
}
