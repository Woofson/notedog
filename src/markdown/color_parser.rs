use crate::theme::parse_color;
use ratatui::style::Color;
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorSpan {
    pub text: String,
    pub fg_color: Option<Color>,
}

static SPAN_STYLE_REGEX: OnceLock<Regex> = OnceLock::new();
static FONT_COLOR_REGEX: OnceLock<Regex> = OnceLock::new();
static BRACKET_COLOR_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_span_regex() -> &'static Regex {
    SPAN_STYLE_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)<span\s+style=["'](?:color:\s*([^"';]+));?["']\s*>(.*?)</span>"#).unwrap()
    })
}

fn get_font_regex() -> &'static Regex {
    FONT_COLOR_REGEX.get_or_init(|| {
        Regex::new(r#"(?i)<font\s+color=["']([^"']+)["']\s*>(.*?)</font>"#).unwrap()
    })
}

fn get_bracket_regex() -> &'static Regex {
    BRACKET_COLOR_REGEX.get_or_init(|| {
        Regex::new(r#"\{\[(#[0-9a-fA-F]{3,6}|[a-zA-Z]+)\](.*?)\}"#).unwrap()
    })
}

pub fn parse_color_tags(input: &str) -> Vec<ColorSpan> {
    if !input.contains('<') && !input.contains("{[") {
        return vec![ColorSpan {
            text: input.to_string(),
            fg_color: None,
        }];
    }

    let mut result = Vec::new();
    let mut cursor = 0;

    // We can iteratively match patterns
    let span_re = get_span_regex();
    let font_re = get_font_regex();
    let bracket_re = get_bracket_regex();

    // Collect all matches with their byte offsets
    #[derive(Debug)]
    struct MatchItem {
        start: usize,
        end: usize,
        color_str: String,
        content: String,
    }

    let mut matches = Vec::new();

    for cap in span_re.captures_iter(input) {
        if let (Some(full), Some(col), Some(txt)) = (cap.get(0), cap.get(1), cap.get(2)) {
            matches.push(MatchItem {
                start: full.start(),
                end: full.end(),
                color_str: col.as_str().to_string(),
                content: txt.as_str().to_string(),
            });
        }
    }

    for cap in font_re.captures_iter(input) {
        if let (Some(full), Some(col), Some(txt)) = (cap.get(0), cap.get(1), cap.get(2)) {
            // Avoid overlaps if already matched
            let start = full.start();
            let end = full.end();
            if !matches.iter().any(|m| (start >= m.start && start < m.end) || (end > m.start && end <= m.end)) {
                matches.push(MatchItem {
                    start,
                    end,
                    color_str: col.as_str().to_string(),
                    content: txt.as_str().to_string(),
                });
            }
        }
    }

    for cap in bracket_re.captures_iter(input) {
        if let (Some(full), Some(col), Some(txt)) = (cap.get(0), cap.get(1), cap.get(2)) {
            let start = full.start();
            let end = full.end();
            if !matches.iter().any(|m| (start >= m.start && start < m.end) || (end > m.start && end <= m.end)) {
                matches.push(MatchItem {
                    start,
                    end,
                    color_str: col.as_str().to_string(),
                    content: txt.as_str().to_string(),
                });
            }
        }
    }

    matches.sort_by_key(|m| m.start);

    for m in matches {
        if m.start > cursor {
            result.push(ColorSpan {
                text: input[cursor..m.start].to_string(),
                fg_color: None,
            });
        }

        let parsed_col = parse_color(&m.color_str);
        result.push(ColorSpan {
            text: m.content,
            fg_color: if parsed_col == Color::Reset { None } else { Some(parsed_col) },
        });

        cursor = m.end;
    }

    if cursor < input.len() {
        result.push(ColorSpan {
            text: input[cursor..].to_string(),
            fg_color: None,
        });
    }

    if result.is_empty() {
        vec![ColorSpan {
            text: input.to_string(),
            fg_color: None,
        }]
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html_span_color() {
        let input = "Hello <span style=\"color:#FF8C00\">Warm Orange</span> text";
        let spans = parse_color_tags(input);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].text, "Hello ");
        assert_eq!(spans[1].text, "Warm Orange");
        assert_eq!(spans[1].fg_color, Some(Color::Rgb(255, 140, 0)));
        assert_eq!(spans[2].text, " text");
    }

    #[test]
    fn test_parse_bracket_shorthand_color() {
        let input = "Text with {[#FFD700]Gold Accent}";
        let spans = parse_color_tags(input);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "Text with ");
        assert_eq!(spans[1].text, "Gold Accent");
        assert_eq!(spans[1].fg_color, Some(Color::Rgb(255, 215, 0)));
    }
}

