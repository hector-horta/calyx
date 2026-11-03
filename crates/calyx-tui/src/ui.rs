use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{ActiveView, TuiApp};
use crate::ui_player::render_player_panel;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(12)])
        .split(frame.area());
    let nav_tab = |label: &'static str, active: bool| -> Span<'static> {
        let style = if active { Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Cyan) };
        Span::styled(label, style)
    };
    let l = app.language;
    let nav_line = Line::from(vec![
        Span::raw("─"),
        nav_tab(l.nav_home(), app.active_view == ActiveView::Home),
        Span::raw(" ─── "),
        nav_tab(l.nav_new(), app.active_view == ActiveView::New),
        Span::raw(" ─── "),
        nav_tab(l.nav_radio(), app.active_view == ActiveView::Radio),
        Span::raw(" ─── "),
        nav_tab(l.nav_library(), app.active_view == ActiveView::Library),
        Span::raw(" ─── "),
        nav_tab(l.nav_playlists(), app.active_view == ActiveView::Playlists),
        Span::raw(" ─── "),
        nav_tab(l.nav_search(), app.is_editing),
        Span::raw(" ─── "),
        nav_tab(l.nav_queue(), app.active_view == ActiveView::Queue),
        Span::raw(" ─── "),
        nav_tab(l.nav_lang(), false),
        Span::raw(" ─── "),
        nav_tab(l.nav_help(), app.active_view == ActiveView::Help),
        Span::raw("───"),
    ]);

    let search_text = if app.is_editing {
        format!(" 🔍 {}█", app.search_input)
    } else if app.search_input.is_empty() {
        l.search_placeholder().to_string()
    } else {
        format!(" 🔍 {}", app.search_input)
    };

    frame.render_widget(
        Paragraph::new(search_text)
            .style(if app.is_editing { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) })
            .block(Block::default().borders(Borders::ALL).title_top(" Calyx Apple Music ")),
        chunks[0],
    );

    match app.active_view {
        ActiveView::Help => crate::ui_help::render_help(frame, chunks[1], app, nav_line),
        _ => render_central_with_lyrics(frame, chunks[1], app, nav_line),
    }
    render_player_panel(frame, chunks[2], app);
}

fn render_central_with_lyrics(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, nav_line: Line<'static>) {
    let bottom_bar = match app.active_view {
        ActiveView::Playlists => match app.language {
            calyx_core::Language::Es => "─[Enter - Abrir Playlist] ─── [p - Reproducir Lista]─",
            calyx_core::Language::En => "─[Enter - Open Playlist] ─── [p - Play Playlist]─",
        },
        ActiveView::Queue => match app.language {
            calyx_core::Language::Es => "─[d - Quitar Pista] ─── [c - Limpiar Cola]─",
            calyx_core::Language::En => "─[d - Remove Track] ─── [c - Clear Queue]─",
        },
        _ => match app.language {
            calyx_core::Language::Es => "─[a - Encolar] ─── [n - A Continuación] ─── [p - Reproducir Lista] ─── [c - Limpiar]─",
            calyx_core::Language::En => "─[a - Enqueue] ─── [n - Play Next] ─── [p - Play Playlist] ─── [c - Clear All]─",
        },
    };
    let outer = Block::default().borders(Borders::ALL).title_top(nav_line).title_bottom(bottom_bar);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(62), Constraint::Percentage(38)]).split(inner);
    match app.active_view {
        ActiveView::Queue => render_queue(frame, cols[0], app),
        ActiveView::Playlists => render_playlists(frame, cols[0], app),
        _ => render_tracks(frame, cols[0], app),
    }
    render_lyrics(frame, cols[1], app);
}

fn render_playlists(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let items: Vec<ListItem<'_>> = if app.playlists.is_empty() {
        vec![ListItem::new(Line::from(Span::styled("  No hay playlists cargadas. Presiona '5' para sincronizar o 'u' para iniciar sesión.", Style::default().fg(Color::DarkGray))))]
    } else {
        app.playlists.iter().map(|p| ListItem::new(Line::from(format!("📁 {:<36} │ {}", truncate(&p.name, 34), truncate(&p.description, 36))))).collect()
    };
    let list = List::new(items)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    let mut state = app.playlist_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}



fn render_queue(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let upcoming = app.queue.upcoming();
    let items: Vec<ListItem<'_>> = if upcoming.is_empty() {
        vec![ListItem::new(Line::from(Span::styled("  La cola está vacía. Selecciona pistas y presiona 'a' o 'n'.", Style::default().fg(Color::DarkGray))))]
    } else {
        upcoming.iter().enumerate().map(|(i, t)| ListItem::new(Line::from(format!("{:>2}. {:<32} │ {:<18} │ {}", i + 1, truncate(&t.title, 30), truncate(&t.artist, 16), t.formatted_duration())))).collect()
    };
    let list = List::new(items)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD));
    let mut state = app.queue_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_tracks(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let items: Vec<ListItem<'_>> = app.tracks.iter().map(|t| ListItem::new(Line::from(format!("{:<32} │ {:<20} │ {:<18} │ {}", truncate(&t.title, 30), truncate(&t.artist, 18), truncate(&t.album, 16), t.formatted_duration())))).collect();
    let list = List::new(items)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    let mut state = app.list_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_lyrics(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let (elap, tot) = app.playback_progress();
    let cur_ms = elap.as_millis() as u64;
    let tot_ms = (tot.as_millis() as u64).max(cur_ms + 4000);
    let active_idx = app.current_lyrics.iter().rposition(|l| l.time_ms <= cur_ms).unwrap_or(0);
    let half = (area.height as usize / 2).max(1);
    let start = active_idx.saturating_sub(half);
    let max_w = area.width.saturating_sub(5) as usize;
    let lines: Vec<Line<'_>> = if app.current_lyrics.is_empty() {
        vec![Line::from(""), Line::from(Span::styled(app.language.no_lyrics(), Style::default().fg(Color::Gray)))]
    } else {
        app.current_lyrics.iter().enumerate().skip(start).take(area.height as usize).map(|(idx, l)| {
            if idx == active_idx {
                let next_ms = app.current_lyrics.get(idx + 1).map(|n| n.time_ms).unwrap_or(tot_ms);
                let (sung, active, unsung) = l.karaoke_parts(cur_ms, next_ms, max_w);
                Line::from(vec![
                    Span::styled(" ♪ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(sung, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(active, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED | Modifier::BOLD)),
                    Span::styled(unsung, Style::default().fg(Color::White)),
                ])
            } else if idx < active_idx {
                Line::from(Span::styled(format!("   {}", truncate(&l.text, max_w)), Style::default().fg(Color::DarkGray)))
            } else {
                Line::from(Span::styled(format!("   {}", truncate(&l.text, max_w)), Style::default().fg(Color::Gray)))
            }
        }).collect()
    };
    frame.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::LEFT).title_top(app.language.title_lyrics())), area);
}

pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars { s.to_string() } else { let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect(); out.push('…'); out }
}