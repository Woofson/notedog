pub mod crypto_dialog;
pub mod editor_view;
pub mod help_dialog;
pub mod note_view;
pub mod notebook_view;
pub mod version_dialog;
pub mod about_dialog;

use crate::app::{App, InputMode, ViewMode};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Fill background if not transparent
    if !app.theme.transparent {
        let bg_block = Block::default().style(app.theme.bg_style());
        f.render_widget(bg_block, size);
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Main content area
            Constraint::Length(1), // Bottom Status / Help bar
        ])
        .split(size);

    let main_area = chunks[0];
    let status_area = chunks[1];

    if !app.is_fullscreen {
        // Main Area Split: Left Sidebar (26%) vs Right Main View (74%)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(26), Constraint::Percentage(74)])
            .split(main_area);

        let sidebar_area = content_chunks[0];
        let main_pane_area = content_chunks[1];

        // Navigation Sidebar (Notebooks + Sections + Notes lists)
        let focused_pane = match app.focused_pane {
            crate::app::Pane::Notebooks => "notebooks",
            crate::app::Pane::Sections => "sections",
            crate::app::Pane::Notes => "notes",
            crate::app::Pane::MainView => "main",
        };

        notebook_view::render_navigation_sidebar(
            f,
            sidebar_area,
            &app.manager,
            app.active_notebook_idx,
            app.active_section_idx,
            app.active_note_idx,
            focused_pane,
            &app.theme,
        );

        // Main Pane: Preview or Built-in Editor
        let is_main_focused = app.focused_pane == crate::app::Pane::MainView;
        match app.view_mode {
            ViewMode::Preview => {
                let note_title = app.current_note_title();
                note_view::render_note_preview(
                    f,
                    main_pane_area,
                    &note_title,
                    &app.current_note_content,
                    app.preview_scroll_y,
                    is_main_focused,
                    app.is_current_note_encrypted(),
                    app.word_wrap,
                    &app.theme,
                );
            }
            ViewMode::Editor => {
                let note_title = app.current_note_title();
                editor_view::render_editor_view(
                    f,
                    main_pane_area,
                    &app.editor,
                    &note_title,
                    is_main_focused,
                    &app.theme,
                );
            }
        }
    } else {
        // Fullscreen Mode: Main Pane fills the entire main viewport!
        let note_title = app.current_note_title();
        match app.view_mode {
            ViewMode::Preview => {
                note_view::render_note_preview(
                    f,
                    main_area,
                    &format!("FULLSCREEN: {}", note_title),
                    &app.current_note_content,
                    app.preview_scroll_y,
                    true,
                    app.is_current_note_encrypted(),
                    app.word_wrap,
                    &app.theme,
                );
            }
            ViewMode::Editor => {
                editor_view::render_editor_view(
                    f,
                    main_area,
                    &app.editor,
                    &format!("FULLSCREEN: {}", note_title),
                    true,
                    &app.theme,
                );
            }
        }
    }

    // 3. Status / Help Bar
    render_status_bar(f, status_area, app, &app.theme);

    // 4. Overlays & Modals
    if app.show_help {
        help_dialog::render_help_modal(f, size, app.help_scroll_y, &app.theme);
    }

    match &app.input_mode {
        InputMode::Normal => {}
        InputMode::PassphrasePrompt { prompt, error } => {
            crypto_dialog::render_passphrase_modal(
                f,
                size,
                prompt,
                &app.input_buffer,
                error.as_deref(),
                &app.theme,
            );
        }
        InputMode::CreateNotebook => {
            crypto_dialog::render_input_dialog(
                f,
                size,
                "New Notebook",
                "Enter Notebook Name:",
                &app.input_buffer,
                None,
                &app.theme,
            );
        }
        InputMode::CreateSection => {
            let default_title = app.config.format_default_section_title();
            crypto_dialog::render_input_dialog(
                f,
                size,
                "New Section",
                "Enter Section Name:",
                &app.input_buffer,
                Some(&default_title),
                &app.theme,
            );
        }
        InputMode::CreateNote => {
            let default_title = app.config.format_default_note_title();
            crypto_dialog::render_input_dialog(
                f,
                size,
                "New Note",
                "Enter Note Title:",
                &app.input_buffer,
                Some(&default_title),
                &app.theme,
            );
        }
        InputMode::RenameNotebook => {
            crypto_dialog::render_input_dialog(
                f,
                size,
                "Rename Notebook",
                "Enter New Notebook Name:",
                &app.input_buffer,
                None,
                &app.theme,
            );
        }
        InputMode::RenameSection => {
            crypto_dialog::render_input_dialog(
                f,
                size,
                "Rename Section",
                "Enter New Section Name:",
                &app.input_buffer,
                None,
                &app.theme,
            );
        }
        InputMode::RenameNote => {
            crypto_dialog::render_input_dialog(
                f,
                size,
                "Rename Note",
                "Enter New Note Title:",
                &app.input_buffer,
                None,
                &app.theme,
            );
        }
        InputMode::ConfirmDelete { title, item_type } => {
            crypto_dialog::render_confirm_delete_modal(
                f,
                size,
                title,
                item_type,
                &app.theme,
            );
        }
        InputMode::VersionHistory => {
            let note_title = app.current_note_title();
            version_dialog::render_version_history_modal(
                f,
                size,
                &app.current_versions,
                app.selected_version_idx,
                app.diff_scroll_y,
                &app.current_note_content,
                &note_title,
                &app.theme,
            );
        }
        InputMode::VersionCleanup => {
            let note_title = app.current_note_title();
            let presets = vec![
                "Keep Last 5 Revisions (Delete older)",
                "Keep Last 10 Revisions (Delete older)",
                "Keep Last 30 Revisions (Delete older)",
                "Keep Revisions Within 30 Days (Delete older)",
                "Purge All History for Current Note",
            ];
            version_dialog::render_version_cleanup_modal(
                f,
                size,
                &presets,
                app.selected_preset_idx,
                &note_title,
                &app.theme,
            );
        }
        InputMode::About => {
            about_dialog::render_about_modal(f, size, &app.theme);
        }
    }
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mode_str = match app.view_mode {
        ViewMode::Preview => " READ ",
        ViewMode::Editor => " EDIT ",
    };

    let mode_color = match app.view_mode {
        ViewMode::Preview => theme.primary, // Ayu Amber Gold
        ViewMode::Editor => theme.secondary, // Ayu Steel Cyan
    };

    let fs_str = if app.is_fullscreen { " ⛶ FULLSCREEN " } else { "" };

    let active_path = format!(
        " 📚 {} / 📂 {} ",
        app.current_notebook_name(),
        app.current_section_name()
    );

    let status_spans = vec![
        Span::styled(
            mode_str,
            Style::default().bg(mode_color).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        if app.is_fullscreen {
            Span::styled(
                fs_str,
                Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("", Style::default())
        },
        Span::styled(active_path, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(" │ {} ", app.status_message),
            Style::default().fg(theme.foreground),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled("[Tab]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" Focus  ", theme.fg_style()),
        Span::styled("[e]", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" Edit  ", theme.fg_style()),
        Span::styled("[Ctrl+N]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" New  ", theme.fg_style()),
        Span::styled("[r]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" Rename  ", theme.fg_style()),
        Span::styled("[?]", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" Help  ", theme.fg_style()),
        Span::styled("[q]", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", theme.fg_style()),
    ];

    let paragraph = Paragraph::new(Line::from(status_spans)).style(theme.bg_style());
    f.render_widget(paragraph, area);
}
