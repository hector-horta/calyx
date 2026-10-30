mod app;
mod artwork;
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

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_panic(info);
    }));

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::TuiApp::new("cl")?;
    let res = run_event_loop(&mut terminal, &mut app).await;
    disable_raw_mode().context("Failed to restore terminal mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    terminal.show_cursor()?;

    res
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::TuiApp,
) -> Result<()> {
    while !app.should_quit {
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
                        KeyCode::Char('a') => app.sync_apple_auth().await,
                        KeyCode::Char('u') => {
                            app.status_message = "Abriendo ventana Apple Sign-In... Inicia sesión y la ventana se cerrará sola.".to_string();
                            terminal.draw(|f| ui::render(f, app))?;
                            app.sync_user_auth().await;
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next_track(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_track(),
                        KeyCode::Enter => {
                            app.status_message = "Iniciando reproducción + Cover Art (Modo Híbrido)...".to_string();
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