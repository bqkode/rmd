mod app;
mod file_tree;
mod markdown;
mod ui;

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use app::{App, AppMode};

#[derive(Parser, Debug)]
#[command(name = "rmd")]
#[command(about = "A terminal-based Markdown document viewer")]
#[command(version)]
struct Args {
    /// Directory to browse for Markdown files (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Resolve the path
    let path = if args.path.is_absolute() {
        args.path
    } else {
        std::env::current_dir()?.join(args.path)
    };

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("Error: '{}' is not a directory", path.display());
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(path);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                match app.mode {
                    AppMode::Search => {
                        // Search mode key handling
                        match key.code {
                            KeyCode::Esc => app.exit_search_mode(),
                            KeyCode::Enter => app.search_select(),
                            KeyCode::Up => app.search_previous(),
                            KeyCode::Down => app.search_next(),
                            KeyCode::Backspace => app.search_backspace(),
                            KeyCode::Char(c) => app.search_add_char(c),
                            _ => {}
                        }
                    }
                    AppMode::Settings => {
                        // Settings mode key handling
                        match key.code {
                            KeyCode::Esc => app.exit_settings_mode(),
                            KeyCode::Enter | KeyCode::Char(' ') => app.settings_toggle_current(),
                            KeyCode::Up => app.settings_previous(),
                            KeyCode::Down => app.settings_next(),
                            _ => {}
                        }
                    }
                    AppMode::About => {
                        // About mode - any key closes it
                        if key.code == KeyCode::Esc || key.code == KeyCode::Enter || key.code == KeyCode::Char('q') {
                            app.mode = AppMode::Normal;
                        }
                    }
                    AppMode::DocumentSearch => {
                        // Document search mode key handling
                        match key.code {
                            KeyCode::Esc => app.exit_doc_search_mode(),
                            KeyCode::Enter => {
                                // Enter goes to next match
                                app.doc_search_next();
                            }
                            KeyCode::Backspace => app.doc_search_backspace(),
                            KeyCode::Char(c) => app.doc_search_add_char(c),
                            _ => {}
                        }
                    }
                    AppMode::Select => {
                        // In select mode, only Esc or v exits back to normal
                        if key.code == KeyCode::Esc || key.code == KeyCode::Char('v') {
                            app.mode = AppMode::Normal;
                            // Re-enable mouse capture
                            execute!(io::stdout(), EnableMouseCapture)?;
                        }
                    }
                    AppMode::Normal => {
                        // Reset pending_g if any other key is pressed (except 'g' itself)
                        if key.code != KeyCode::Char('g') && app.pending_g {
                            app.pending_g = false;
                        }

                        // Quit on q, Esc, or Ctrl+c
                        if key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Esc
                            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            return Ok(());
                        }

                        match key.code {
                            // Navigation
                            KeyCode::Char('j') | KeyCode::Down => app.next(),
                            KeyCode::Char('k') | KeyCode::Up => app.previous(),
                            KeyCode::Char('l') | KeyCode::Right => app.focus_content_or_select(),
                            KeyCode::Char('h') | KeyCode::Left => app.focus_sidebar_or_collapse(),
                            KeyCode::Enter => app.toggle_or_select(),
                            KeyCode::Tab => app.toggle_focus(),

                            // gg for top, G for bottom
                            KeyCode::Char('g') => {
                                if app.pending_g {
                                    app.scroll_to_top();
                                    app.pending_g = false;
                                } else {
                                    app.pending_g = true;
                                }
                            }
                            KeyCode::Char('G') => app.scroll_to_bottom(),

                            // Vim-style page navigation
                            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+u: half page up
                                let half_page = (app.content_height / 2) as usize;
                                app.content_scroll = app.content_scroll.saturating_sub(half_page);
                            }
                            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+d: half page down
                                let half_page = (app.content_height / 2) as usize;
                                let max_scroll = app.total_wrapped_lines().saturating_sub(app.content_height as usize);
                                app.content_scroll = (app.content_scroll + half_page).min(max_scroll);
                            }
                            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+b: full page up
                                let page_size = app.content_height as usize;
                                app.content_scroll = app.content_scroll.saturating_sub(page_size);
                            }
                            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+f: full page down
                                let page_size = app.content_height as usize;
                                let max_scroll = app.total_wrapped_lines().saturating_sub(app.content_height as usize);
                                app.content_scroll = (app.content_scroll + page_size).min(max_scroll);
                            }

                            // Search
                            KeyCode::Char('/') => {
                                // /: search in document (vim style)
                                app.enter_doc_search_mode();
                            }
                            KeyCode::Char('n') => {
                                // n: next search result
                                app.doc_search_next();
                            }
                            KeyCode::Char('N') => {
                                // N: previous search result
                                app.doc_search_previous();
                            }

                            // Global search (custom, no vim equivalent)
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.enter_search_mode();
                            }

                            // Settings (custom)
                            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.enter_settings_mode();
                            }

                            // About window
                            KeyCode::Char('?') => {
                                app.mode = AppMode::About;
                            }

                            // Select mode for text selection
                            KeyCode::Char('v') => {
                                app.mode = AppMode::Select;
                                // Disable mouse capture to allow terminal text selection
                                execute!(io::stdout(), DisableMouseCapture)?;
                            }

                            // Fallback keys
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Home => app.scroll_content_to_top(),
                            KeyCode::End => app.scroll_content_to_bottom(),
                            _ => {}
                        }
                    }
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        // Scroll content up 3 lines
                        app.content_scroll = app.content_scroll.saturating_sub(3);
                    }
                    MouseEventKind::ScrollDown => {
                        // Scroll content down 3 lines
                        let max_scroll = app.total_wrapped_lines().saturating_sub(app.content_height as usize);
                        app.content_scroll = (app.content_scroll + 3).min(max_scroll);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
