mod app;
mod artwork;
mod playback;
mod ui;

use std::io;
use std::time::Duration;
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--check" || a == "--version") {
        return Ok(());
    }

    let storefront = args.get(1).map(|s| s.as_str()).unwrap_or("us");
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create Ratatui terminal")?;

    let mut app = app::TuiApp::new(storefront).context("Failed to initialize TuiApp")?;
    let res = run_event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Calyx encountered an error: {err:?}");
    }
    Ok(())
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::TuiApp,
) -> Result<()> {
    while !app.should_quit {
        app.check_auto_advance().await;
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                if app.is_editing {
                    match key_event.code {
                        KeyCode::Enter => {
                            app.is_editing = false;
                            app.execute_search().await;
                        }
                        KeyCode::Esc => app.is_editing = false,
                        KeyCode::Backspace => { let _ = app.search_input.pop(); },
                        KeyCode::Char(c) => app.search_input.push(c),
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('/') => app.is_editing = true,
                        KeyCode::Char('t') => app.sync_apple_auth().await,
                        KeyCode::Char('a') => app.enqueue_selected_back(),
                        KeyCode::Char('n') => app.enqueue_selected_next(),
                        KeyCode::Char('d') => app.remove_selected_from_queue(),
                        KeyCode::Char('s') => { app.toggle_shuffle(); },
                        KeyCode::Char('r') => { app.toggle_repeat(); },
                        KeyCode::Tab => app.toggle_view(),
                        KeyCode::Char('u') => {
                            app.status_message = "Abriendo ventana Apple Sign-In... Inicia sesión y la ventana se cerrará sola.".to_string();
                            terminal.draw(|f| ui::render(f, app))?;
                            app.sync_user_auth().await;
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next_track(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_track(),
                        KeyCode::Enter => {
                            terminal.draw(|f| ui::render(f, app))?;
                            app.play_selected().await;
                        }
                        KeyCode::Char(' ') => app.toggle_pause().await,
                        KeyCode::Char('+') | KeyCode::Char('=') => app.adjust_volume(0.05).await,
                        KeyCode::Char('-') => app.adjust_volume(-0.05).await,
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
