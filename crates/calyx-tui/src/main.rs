mod activity;
mod album_view;
mod app;
mod artwork;
mod audio_pipeline;
mod bg_events;
mod cli;
mod hints;
mod labels;
mod list_actions;
mod list_kind;
mod login;
mod messages;
mod messages_picker;
mod navigation;
mod notice;
mod notice_tone;
mod picker;
mod picker_texts;
mod playback;
mod player_big_title;
mod player_spectrum;
mod player_state;
mod player_status;
mod player_track;
mod playlist_remove;
mod spinner;
mod tasks;
mod ui;
mod ui_dancer;
mod ui_help;
mod ui_lists;
mod ui_lyrics;
mod ui_picker;
mod ui_player;
mod update_install;
mod version_badge;

use anyhow::{Context, Result};
use calyx_core::SectionType;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = cli::Cli::parse(&args);
    if cli.version {
        println!("calyx {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if cli.check {
        return Ok(());
    }
    if let Some(name) = &cli.login {
        let Some(browser) = calyx_core::browser_choice::named(name) else {
            let known = calyx_core::browser_choice::known_names().join(", ");
            anyhow::bail!("browser '{name}' is unknown or not installed (options: {known})");
        };
        calyx_core::browser_choice::prefer(browser);
    }
    // Built before raw mode, so a startup error prints on a normal terminal.
    let mut app = app::TuiApp::new(&cli.storefront).context("Failed to initialize TuiApp")?;
    // The binary replaced by the last update is no longer running.
    calyx_core::self_update::remove_old_binary();
    if cli.login.is_some() {
        app.begin_login();
    }
    if cli.update_check {
        version_badge::start_check();
    }

    install_panic_hook();
    enable_raw_mode().context("Failed to enable raw mode")?;
    let res = match execute!(io::stdout(), EnterAlternateScreen)
        .context("Failed to enter alternate screen")
        .and_then(|_| {
            Terminal::new(CrosstermBackend::new(io::stdout()))
                .context("Failed to create Ratatui terminal")
        }) {
        Ok(mut terminal) => run_event_loop(&mut terminal, &mut app),
        Err(e) => Err(e),
    };
    restore_terminal();

    if let Err(err) = res {
        eprintln!("Calyx encountered an error: {err:?}");
    }
    Ok(())
}

/// Leaves raw mode and the alternate screen; errors are ignored (best effort).
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
}

/// Restores the terminal before the default panic message is printed.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        previous(info);
    }));
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::TuiApp,
) -> Result<()> {
    // Nothing in this loop awaits the network: remote work runs on tasks
    // (see tasks.rs) and comes back through poll_bg_events.
    while !app.should_quit {
        app.poll_bg_events();
        app.check_auto_advance();
        terminal.draw(|f| ui::render(f, app))?;

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let Event::Key(key_event) = event::read()? else {
            continue;
        };
        if key_event.kind != KeyEventKind::Press {
            continue;
        }
        app.dismiss_key_notice();

        if app.is_editing {
            match key_event.code {
                KeyCode::Enter => {
                    app.is_editing = false;
                    app.start_search();
                }
                KeyCode::Esc => app.is_editing = false,
                KeyCode::Backspace => {
                    app.search_input.pop();
                }
                KeyCode::Char(c) => app.search_input.push(c),
                _ => {}
            }
            continue;
        }

        if app.picker.is_some() {
            app.picker_key(key_event.code);
            continue;
        }

        match key_event.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('V') => app.update_key(),
            KeyCode::Char('m') => app.open_playlist_picker(),
            KeyCode::Char('/') => app.is_editing = true,
            KeyCode::Char('a') => app.enqueue_selected_back(),
            KeyCode::Char('A') => app.enqueue_visible_list(),
            KeyCode::Char('o') => app.toggle_album(),
            KeyCode::Char('f') => app.add_selected_album_to_library(),
            KeyCode::Char('n') => app.enqueue_selected_next(),
            KeyCode::Char('d') => app.remove_selected_from_queue(),
            KeyCode::Char('c') => app.clear_current_list(),
            KeyCode::Char('y') | KeyCode::Char('Y') => app.toggle_phonetic_mode(),
            KeyCode::Char('s') => {
                app.toggle_shuffle();
            }
            KeyCode::Char('r') => {
                app.toggle_repeat();
            }
            KeyCode::Char('l') | KeyCode::Char('L') => app.toggle_language(),
            KeyCode::Char('1') => app.open_home(),
            KeyCode::Char('2') => app.open_section(
                app::ActiveView::New,
                SectionType::NewReleases,
                messages::ListKind::NewReleases,
            ),
            KeyCode::Char('3') => app.open_section(
                app::ActiveView::Radio,
                SectionType::Radio,
                messages::ListKind::Radio,
            ),
            KeyCode::Char('4') => app.sync_library(),
            KeyCode::Char('5') => app.sync_playlists(),
            KeyCode::Char('p') | KeyCode::Char('P') => app.play_all(),
            KeyCode::Char('?') => {
                app.active_view = app::ActiveView::Help;
                app.say(messages::Msg::HelpOpened);
            }
            KeyCode::Char('>') | KeyCode::Char('.') => app.skip_next(),
            KeyCode::Char('<') | KeyCode::Char(',') => app.play_previous(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Char('u') => app.begin_login(),
            KeyCode::Down | KeyCode::Char('j') => app.next_track(),
            KeyCode::Up | KeyCode::Char('k') => app.previous_track(),
            KeyCode::Enter => app.play_selected(),
            KeyCode::Char(' ') => app.toggle_pause(),
            KeyCode::Char('+') | KeyCode::Char('=') => app.adjust_volume(0.05),
            KeyCode::Char('-') => app.adjust_volume(-0.05),
            _ => {}
        }
    }
    Ok(())
}
