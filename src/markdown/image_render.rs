use std::path::{Path, PathBuf};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use crate::theme::Theme;

pub fn parse_image_tag(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("![") {
        return None;
    }
    let close_bracket = trimmed.find(']')?;
    let alt = &trimmed[2..close_bracket];
    let rest = trimmed[close_bracket + 1..].trim();
    if !rest.starts_with('(') || !rest.ends_with(')') {
        return None;
    }
    let path = rest[1..rest.len() - 1].trim();
    Some((alt, path))
}

pub fn render_image_to_lines(
    path_str: &str,
    alt: &str,
    base_dir: Option<&Path>,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // 1. Check for remote URL
    if path_str.starts_with("http://") || path_str.starts_with("https://") {
        lines.push(Line::from(vec![
            Span::styled("┌── 🌐 Remote Image: ", Style::default().fg(theme.primary)),
            Span::styled(alt.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" ───────────────────────┐", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("│ URL: ", Style::default().fg(theme.border)),
            Span::styled(path_str.to_string(), Style::default().fg(theme.secondary)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("└──────────────────────────────────────────────┘", Style::default().fg(theme.border)),
        ]));
        return lines;
    }

    // 2. Resolve local file path
    let resolved_path = resolve_image_path(path_str, base_dir);
    let filename = Path::new(path_str)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.to_string());

    let alt_display = if alt.trim().is_empty() { filename.as_str() } else { alt };

    if !resolved_path.exists() {
        lines.push(Line::from(vec![
            Span::styled("┌── 🖼️ Image Not Found: ", Style::default().fg(Color::LightRed)),
            Span::styled(alt_display.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" ───────────────────┐", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("│ Path: ", Style::default().fg(theme.border)),
            Span::styled(resolved_path.to_string_lossy().to_string(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("└──────────────────────────────────────────────┘", Style::default().fg(theme.border)),
        ]));
        return lines;
    }

    // 3. Load and decode image
    let dyn_img = match image::open(&resolved_path) {
        Ok(img) => img,
        Err(e) => {
            lines.push(Line::from(vec![
                Span::styled("┌── ⚠️ Image Decode Error: ", Style::default().fg(Color::Yellow)),
                Span::styled(alt_display.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(" ────────────────┐", Style::default().fg(theme.border)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("│ Error: ", Style::default().fg(theme.border)),
                Span::styled(e.to_string(), Style::default().fg(Color::Red)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("└──────────────────────────────────────────────┘", Style::default().fg(theme.border)),
            ]));
            return lines;
        }
    };

    let (orig_w, orig_h) = (dyn_img.width(), dyn_img.height());

    // 4. Determine scaled dimensions
    // In typical terminal fonts, a cell is ~2:1 height:width ratio.
    // Each halfblock char `▀` has 2 vertical pixels.
    // So 1 character cell is 1 horizontal pixel and 2 vertical pixels in our half-block grid.
    let max_cell_width = 54u32;
    let cell_w = orig_w.min(max_cell_width).max(1);
    let scale_ratio = cell_w as f32 / orig_w.max(1) as f32;
    let cell_h = ((orig_h as f32 * scale_ratio) * 0.5).round().max(1.0) as u32;
    let pixel_h = cell_h * 2;

    let resized = dyn_img.resize_exact(cell_w, pixel_h, image::imageops::FilterType::Triangle);
    let rgba_img = resized.to_rgba8();

    // 5. Header Frame
    let header_label = format!(" 🖼️ {} ({}x{}) ", alt_display, orig_w, orig_h);
    lines.push(Line::from(vec![
        Span::styled("┌──", Style::default().fg(theme.border)),
        Span::styled(header_label, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("────────────────────────", Style::default().fg(theme.border)),
    ]));

    // 6. Half-Block Pixel Rows
    for cy in 0..cell_h {
        let mut spans = Vec::with_capacity(cell_w as usize + 2);
        spans.push(Span::styled("│ ", Style::default().fg(theme.border)));

        for cx in 0..cell_w {
            let top_pixel = rgba_img.get_pixel(cx, cy * 2);
            let bot_pixel = rgba_img.get_pixel(cx, cy * 2 + 1);

            if top_pixel[3] < 30 && bot_pixel[3] < 30 {
                spans.push(Span::raw(" "));
            } else if top_pixel[3] < 30 {
                spans.push(Span::styled(
                    "▄",
                    Style::default().fg(Color::Rgb(bot_pixel[0], bot_pixel[1], bot_pixel[2])),
                ));
            } else if bot_pixel[3] < 30 {
                spans.push(Span::styled(
                    "▀",
                    Style::default().fg(Color::Rgb(top_pixel[0], top_pixel[1], top_pixel[2])),
                ));
            } else {
                spans.push(Span::styled(
                    "▀",
                    Style::default()
                        .fg(Color::Rgb(top_pixel[0], top_pixel[1], top_pixel[2]))
                        .bg(Color::Rgb(bot_pixel[0], bot_pixel[1], bot_pixel[2])),
                ));
            }
        }

        spans.push(Span::styled(" │", Style::default().fg(theme.border)));
        lines.push(Line::from(spans));
    }

    // 7. Footer Frame
    lines.push(Line::from(vec![
        Span::styled("└──────────────────────────────────────────────┘", Style::default().fg(theme.border)),
    ]));

    lines
}

fn resolve_image_path(path_str: &str, base_dir: Option<&Path>) -> PathBuf {
    if path_str.starts_with("~/") || path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            } else {
                return home.join(&path_str[2..]);
            }
        }
    }

    let p = Path::new(path_str);
    if p.is_absolute() {
        return p.to_path_buf();
    }

    if let Some(base) = base_dir {
        let candidate = base.join(p);
        if candidate.exists() {
            return candidate;
        }
    }

    p.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_tag() {
        assert_eq!(
            parse_image_tag("![Alt Text](photo.png)"),
            Some(("Alt Text", "photo.png"))
        );
        assert_eq!(
            parse_image_tag("  ![Diagram](./assets/diag.jpg)  "),
            Some(("Diagram", "./assets/diag.jpg"))
        );
        assert_eq!(
            parse_image_tag("![](https://example.com/logo.webp)"),
            Some(("", "https://example.com/logo.webp"))
        );
        assert_eq!(parse_image_tag("Just text with ![not an image]"), None);
    }
}
