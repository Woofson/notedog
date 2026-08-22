use std::path::Path;
use crate::markdown::color_parser::parse_color_tags;
use crate::mermaid::{parse_mermaid, render_mermaid_to_lines};
use crate::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render_markdown(
    markdown_text: &str,
    theme: &Theme,
    base_dir: Option<&Path>,
    icons: Option<&crate::config::IconConfig>,
) -> Vec<Line<'static>> {
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
            let h1_icon = icons.map(|ic| ic.header_1.as_str()).unwrap_or("📌 ");
            let mut line_spans = Vec::new();
            if !h1_icon.is_empty() {
                line_spans.push(Span::styled(h1_icon.to_string(), Style::default().fg(theme.primary)));
            }
            line_spans.push(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(theme.header_1)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
            lines.push(Line::from(line_spans));
            lines.push(Line::from(""));
            continue;
        } else if line.starts_with("## ") {
            let text = &line[3..];
            let h2_icon = icons.map(|ic| ic.header_2.as_str()).unwrap_or("🔸 ");
            let mut line_spans = Vec::new();
            if !h2_icon.is_empty() {
                line_spans.push(Span::styled(h2_icon.to_string(), Style::default().fg(theme.secondary)));
            }
            line_spans.push(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(theme.header_2)
                    .add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(line_spans));
            lines.push(Line::from(""));
            continue;
        } else if line.starts_with("### ") {
            let text = &line[4..];
            let h3_icon = icons.map(|ic| ic.header_3.as_str()).unwrap_or("🔹 ");
            let mut line_spans = Vec::new();
            if !h3_icon.is_empty() {
                line_spans.push(Span::styled(h3_icon.to_string(), Style::default().fg(theme.accent)));
            }
            line_spans.push(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(theme.header_3)
                    .add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(line_spans));
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

        // Markdown Image: ![Alt text](path/to/image.png)
        if let Some((alt, img_path)) = crate::markdown::image_render::parse_image_tag(trimmed) {
            let img_lines = crate::markdown::image_render::render_image_to_lines(img_path, alt, base_dir, theme);
            lines.extend(img_lines);
            lines.push(Line::from(""));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_with_image() {
        let theme = Theme::from_config(&crate::theme::ThemeConfig::default(), true);
        let md = "# Title\n\n![Test Image](non_existent_test_image.png)\n\nSome text.";
        let lines = render_markdown(md, &theme, None, None);
        assert!(!lines.is_empty());
        let full_text: String = lines.iter().map(|l| {
            l.spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>().join("")
        }).collect::<Vec<_>>().join("\n");
        assert!(full_text.contains("Test Image") || full_text.contains("Image Not Found"));
    }

    #[test]
    fn test_custom_header_icons() {
        let theme = Theme::from_config(&crate::theme::ThemeConfig::default(), true);
        let mut icons = crate::config::IconConfig::default();
        icons.header_1 = "🔖 ".to_string();
        icons.header_2 = "⭐ ".to_string();
        let md = "# Main Header\n\n## Sub Header";
        let lines = render_markdown(md, &theme, None, Some(&icons));
        let full_text: String = lines.iter().map(|l| {
            l.spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>().join("")
        }).collect::<Vec<_>>().join("\n");
        assert!(full_text.contains("🔖 Main Header"));
        assert!(full_text.contains("⭐ Sub Header"));
    }
}

