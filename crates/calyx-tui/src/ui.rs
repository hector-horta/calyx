use crate::app::{ActiveView, PhoneticMode, TuiApp};
use crate::hints::{LABEL_STYLE, hint, hint_bar, hint_bar_fit, tab, tab_bar};
use crate::ui_lists::{render_playlists, render_queue, render_tracks};
use crate::ui_player::render_player_panel;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(12),
        ])
        .split(frame.area());
    let l = app.language;
    let nav_line = tab_bar(vec![
        tab("1", l.nav_home(), app.active_view == ActiveView::Home),
        tab("2", l.nav_new(), app.active_view == ActiveView::New),
        tab("3", l.nav_radio(), app.active_view == ActiveView::Radio),
        tab("4", l.nav_library(), app.active_view == ActiveView::Library),
        tab(
            "5",
            l.nav_playlists(),
            app.active_view == ActiveView::Playlists,
        ),
        tab("Tab", l.nav_queue(), app.active_view == ActiveView::Queue),
        tab("L", l.nav_lang(), false),
    ]);

    let search_text = if app.is_editing {
        format!(" 🔍 {}█", app.search_input)
    } else if app.search_input.is_empty() {
        l.search_placeholder().to_string()
    } else {
        format!(" 🔍 {}", app.search_input)
    };

    let mut top_actions = Vec::new();
    if !app.is_logged_in() {
        top_actions.push(hint(
            "u",
            l.nav_login(),
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }
    top_actions.push(tab("?", l.nav_help(), app.active_view == ActiveView::Help));
    let help_line = tab_bar(top_actions).alignment(Alignment::Right);

    let top_block = Block::default()
        .borders(Borders::ALL)
        .title_top(crate::version_badge::title(app.language, app.update))
        .title_top(help_line);

    frame.render_widget(
        Paragraph::new(search_text)
            .style(if app.is_editing {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            })
            .block(top_block),
        chunks[0],
    );

    match app.active_view {
        ActiveView::Help => crate::ui_help::render_help(frame, chunks[1], app, nav_line),
        _ => render_central_with_lyrics(frame, chunks[1], app, nav_line),
    }
    render_player_panel(frame, chunks[2], app);
    crate::ui_picker::render_picker(frame, app);
}

fn render_central_with_lyrics(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &TuiApp,
    nav_line: Line<'static>,
) {
    let l = app.language;
    let phonetic = match app.phonetic_mode {
        PhoneticMode::Dual => l.btn_phonetic_dual(),
        PhoneticMode::Phonetic => l.btn_phonetic_only(),
        PhoneticMode::Original => l.btn_phonetic_off(),
    };
    let yellow = Style::new().fg(Color::Yellow);
    let bottom_bar = match app.active_view {
        ActiveView::Playlists => hint_bar(&[
            ("Enter", l.btn_open(), LABEL_STYLE),
            ("p", l.btn_play_list(), LABEL_STYLE),
        ]),
        ActiveView::Queue => hint_bar(&[
            ("d", l.btn_remove_track(), LABEL_STYLE),
            ("y", phonetic, yellow),
            ("c", l.btn_clear_all(), LABEL_STYLE),
        ]),
        // Most used first: in narrow terminals the last hints are dropped.
        _ => {
            let mut items = vec![
                ("a", l.btn_enqueue(), LABEL_STYLE),
                ("n", l.btn_next_in_queue(), LABEL_STYLE),
                ("p", l.btn_play_list(), LABEL_STYLE),
                ("o", album_key_label(app), LABEL_STYLE),
                ("m", crate::labels::to_playlist(l), LABEL_STYLE),
                ("f", crate::labels::album_to_library(l), LABEL_STYLE),
                ("y", phonetic, yellow),
                ("c", l.btn_clear_all(), LABEL_STYLE),
            ];
            if app.can_remove_from_open_playlist() {
                items.insert(
                    5,
                    ("d", crate::labels::remove_from_playlist(l), LABEL_STYLE),
                );
            }
            hint_bar_fit(&items, area.width.saturating_sub(2) as usize)
        }
    };
    // List messages replace the key hints while they are relevant.
    let width = area.width.saturating_sub(2) as usize;
    let bottom = crate::zones::list_line(&app.zones.list, &app.activity, width);
    let outer = Block::default()
        .borders(Borders::ALL)
        .title_top(nav_line)
        .title_bottom(bottom.unwrap_or(bottom_bar));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(inner);
    match app.active_view {
        ActiveView::Queue => render_queue(frame, cols[0], app),
        ActiveView::Playlists => render_playlists(frame, cols[0], app),
        _ => render_tracks(frame, cols[0], app),
    }
    crate::ui_lyrics::render_lyrics(frame, cols[1], app);
}

/// "Álbum", or "← Biblioteca" (where 'o' goes back to) while an album is shown.
fn album_key_label(app: &TuiApp) -> &'static str {
    match app.album_back.as_ref().filter(|_| app.showing_album()) {
        Some(saved) => crate::labels::back_to(app.language, saved.view),
        None => crate::labels::album(app.language),
    }
}

pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
