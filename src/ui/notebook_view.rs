use crate::note_manager::NoteManager;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Tabs},
    Frame,
};

pub fn render_notebook_tabs(
    f: &mut Frame,
    area: Rect,
    manager: &NoteManager,
    active_notebook_idx: usize,
    is_focused: bool,
    theme: &Theme,
) {
    let titles: Vec<Line> = manager
        .notebooks
        .iter()
        .enumerate()
        .map(|(i, nb)| {
            let prefix = if i == active_notebook_idx { "📁 " } else { "📁 " };
            Line::from(format!("{}{}", prefix, nb.name))
        })
        .collect();

    let title_str = if is_focused { " ▶ 📚 NOTEBOOKS ◀ " } else { " 📚 NOTEBOOKS " };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(if is_focused {
                    theme.active_border_style()
                } else {
                    theme.border_style()
                })
                .title(if is_focused {
                    Span::styled(title_str, theme.active_title_style())
                } else {
                    Span::styled(title_str, theme.title_style())
                }),
        )
        .select(active_notebook_idx)
        .style(theme.tab_inactive_style())
        .highlight_style(theme.tab_active_style());

    f.render_widget(tabs, area);
}

pub fn render_navigation_sidebar(
    f: &mut Frame,
    area: Rect,
    manager: &NoteManager,
    active_nb: usize,
    active_sec: usize,
    active_note: usize,
    focused_pane: &str, // "notebooks", "sections", "notes"
    theme: &Theme,
    icons: &crate::config::IconConfig,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(26), // Notebooks
            Constraint::Percentage(34), // Sections
            Constraint::Percentage(40), // Notes
        ])
        .split(area);

    // 1. Notebooks List
    let nb_items: Vec<ListItem> = manager
        .notebooks
        .iter()
        .enumerate()
        .map(|(i, nb)| {
            let is_selected = i == active_nb;
            let icon = &icons.notebook;
            let style = if is_selected {
                theme.highlight_style()
            } else {
                theme.fg_style()
            };
            ListItem::new(Line::from(vec![
                Span::styled(icon.as_str(), Style::default().fg(theme.sidebar_title)),
                Span::styled(nb.name.clone(), style),
            ]))
        })
        .collect();

    let is_nb_focused = focused_pane == "notebooks";
    let nb_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.notebook_border_style(is_nb_focused))
        .title(Span::styled(format!(" {} NOTEBOOKS ", icons.notebook.trim()), theme.notebook_title_style(is_nb_focused)));

    let nb_list = List::new(nb_items).block(nb_block);
    f.render_widget(nb_list, chunks[0]);

    let current_nb = manager.notebooks.get(active_nb);

    // 2. Sections List
    let sec_items: Vec<ListItem> = if let Some(nb) = current_nb {
        nb.sections
            .iter()
            .enumerate()
            .map(|(i, sec)| {
                let is_selected = i == active_sec;
                let icon = &icons.section;
                let style = if is_selected {
                    theme.highlight_style()
                } else {
                    theme.fg_style()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(icon.as_str(), Style::default().fg(theme.sidebar_title)),
                    Span::styled(sec.name.clone(), style),
                ]))
            })
            .collect()
    } else {
        vec![ListItem::new(" (No sections)")]
    };

    let is_sec_focused = focused_pane == "sections";
    let sec_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.section_border_style(is_sec_focused))
        .title(Span::styled(format!(" {} SECTIONS ", icons.section.trim()), theme.section_title_style_comp(is_sec_focused)));

    let sec_list = List::new(sec_items).block(sec_block);
    f.render_widget(sec_list, chunks[1]);

    // 3. Notes List
    let current_sec = current_nb.and_then(|nb| nb.sections.get(active_sec));
    let note_items: Vec<ListItem> = if let Some(sec) = current_sec {
        sec.notes
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let is_selected = i == active_note;
                let icon = if note.is_encrypted { &icons.encrypted_note } else { &icons.note };
                let name_style = if is_selected {
                    theme.highlight_style()
                } else if note.is_encrypted {
                    Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)
                } else {
                    theme.fg_style()
                };

                let lock_badge = if note.is_encrypted { " [ENC]" } else { "" };
                ListItem::new(Line::from(vec![
                    Span::styled(icon.as_str(), Style::default().fg(if note.is_encrypted { theme.encrypted_tag } else { theme.sidebar_title })),
                    Span::styled(note.name.clone(), name_style),
                    Span::styled(lock_badge, Style::default().fg(theme.encrypted_tag)),
                ]))
            })
            .collect()
    } else {
        vec![ListItem::new(" (No notes)")]
    };

    let is_note_focused = focused_pane == "notes";
    let note_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.note_border_style(is_note_focused))
        .title(Span::styled(format!(" {} NOTES ", icons.note.trim()), theme.note_title_style(is_note_focused)));

    let note_list = List::new(note_items).block(note_block);
    f.render_widget(note_list, chunks[2]);
}
