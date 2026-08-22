#![allow(dead_code)]

mod app;
mod config;
mod crypto;
mod editor;
mod markdown;
mod mermaid;
mod note_manager;
mod theme;
mod ui;
mod versioning;
mod diff;

use app::App;
use config::Config;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("🐶 NoteDog v{} - OneNote & Obsidian style TUI notes app", env!("CARGO_PKG_VERSION"));
        println!("Author: Bolt J Woofson | Repository: https://github.com/Woofson/notedog");
        println!();
        println!("USAGE:");
        println!("  notedog [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  -c, --clean     Initialize/verify clean notes system without overwriting existing notes");
        println!("  -v, --version   Print NoteDog version & information");
        println!("  -h, --help      Print this help menu");
        println!();
        println!("CONFIGURATION & STORAGE:");
        println!("  Config path:  ~/.config/notedog/notedog.toml");
        println!("  Notes folder: ~/Notes (or configured note_folder)");
        return Ok(());
    }

    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("NoteDog v{} (Author: Bolt J Woofson, Repository: Woofson/notedog)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Install panic hook to ensure terminal state is restored if any panic happens
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_panic(panic_info);
    }));

    let (cfg, _config_path) = Config::load_or_create();

    if args.contains(&"--clean".to_string()) || args.contains(&"-c".to_string()) {
        let note_folder = cfg.resolved_note_folder();
        println!("🐶 NoteDog Clean Notes Initializer");
        println!("Notes directory: {:?}", note_folder);

        if note_folder.exists() {
            println!("[SAFEGUARD] Existing notes directory found at {:?}.", note_folder);
            println!("[SAFEGUARD] Preserving all existing notebooks, sections, and note files!");
        }

        let _ = note_manager::NoteManager::new(note_folder);
        println!("✅ Clean starter notes system verified & ready!");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(cfg);

    let run_res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = run_res {
        println!("Notedog exited with error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| ui::render_ui(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == event::KeyEventKind::Press {
                    let should_quit = app.handle_key_event(key_event);
                    if should_quit {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
