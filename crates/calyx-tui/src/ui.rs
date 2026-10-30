use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::TuiApp;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(12),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(app.search_input.as_str())
            .style(
                if app.is_editing {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Cyan)
                }
            )
            .block(Block::default()
                .borders(Borders::ALL)
                .title(
                    if app.is_editing {
                        " Buscar en Apple Music (Escribe y presiona Enter | Esc para cancelar) "
                    } else {
                        " Calyx TUI — Presiona '/' para buscar | 'j/k' o Flechas navegar | 'Enter' reproducir "
                    }
                )),
        chunks[0]
    );

    let items: Vec<ListItem<'_>> = app
        .tracks
        .iter()
        .enumerate()
        .map(|(idx, track)| {
            ListItem::new(
                Line::from(Span::styled(
                    format!(
                        "{}{:<32} │ {:<22} │ {:<20} │ {} {}",
                        if idx == app.selected_index { "▶ " } else { "  " },
                        truncate(&track.title, 30),
                        truncate(&track.artist, 20),
                        truncate(&track.album, 18),
                        track.formatted_duration(),
                        if track.is_lossless { "[ALAC/AAC]" } else { "[AAC]" }
                    ),
                    if idx == app.selected_index {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    }
                ))
            )
        })
        .collect();

    frame.render_widget(
        List::new(items).block(Block::default()
            .borders(Borders::ALL)
            .title(" Catálogo Apple Music & Caché SQLite WAL ")),
        chunks[1]
    );

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(30)])
        .split(chunks[2]);

    let (art_lines, art_title) = match &app.current_artwork {
        Some(art) => (
            art.to_lines(),
            if art.is_real_image { " Cover Art [JPEG] " } else { " Cover Art [Synth] " },
        ),
        None => (
            crate::artwork::TerminalCoverArt::from_jpeg_or_seed(None, "Calyx").to_lines(),
            " Cover Art ",
        ),
    };
    let art_widget = Paragraph::new(art_lines)
        .block(Block::default().borders(Borders::ALL).title(art_title));
    frame.render_widget(art_widget, bottom_chunks[0]);

    let auth_badge = if app.session.has_full_stream_credentials() {
        "Full DRM (Built-in L3 Ready)"
    } else if app.session.developer_token.is_some() {
        "DevToken OK (Presiona 'u' para Login Apple)"
    } else {
        "Preview Mode ('a' DevToken | 'u' Login Apple)"
    };

    let now_playing = match &app.current_track {
        Some(t) => format!(
            "Estado: {:?} │ {} — {} ({})",
            app.playback_status(),
            t.artist,
            t.title,
            t.formatted_duration(),
        ),
        None => format!("Estado: {:?} │ Sin pista activa", app.playback_status()),
    };

    let meta_line = format!(
        "Volumen: {:>3}% │ Modo: {} │ Pipeline: {:?}",
        (app.volume * 100.0) as u8,
        auth_badge,
        app.drm_mode
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(now_playing, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(meta_line, Style::default().fg(Color::Cyan))),
            Line::from(Span::raw("")),
            Line::from(Span::styled(&app.status_message, Style::default().fg(Color::Gray))),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Motor de Audio Nativo (Symphonia + Rodio/CPAL + Widevine L3) ")),
        bottom_chunks[1]
    );
}

fn truncate(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}