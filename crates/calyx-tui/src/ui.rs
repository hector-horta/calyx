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
                        " Calyx — '/' Buscar │ '↑/↓' o 'j/k' Navegar │ 'Enter' Reproducir │ 'Espacio' Pausa │ 'q' Salir "
                    }
                )),
        chunks[0]
    );

    let items: Vec<ListItem<'_>> = app
        .tracks
        .iter()
        .map(|track| {
            ListItem::new(Line::from(Span::styled(
                format!(
                    "{:<38} │ {:<24} │ {:<22} │ {}",
                    truncate(&track.title, 36),
                    truncate(&track.artist, 22),
                    truncate(&track.album, 20),
                    track.formatted_duration(),
                ),
                Style::default().fg(Color::White),
            )))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Catálogo Apple Music "))
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    let mut state = app.list_state.clone();
    frame.render_stateful_widget(list, chunks[1], &mut state);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(30)])
        .split(chunks[2]);

    let art_lines = match &app.current_artwork {
        Some(art) => art.to_lines(),
        None => crate::artwork::TerminalCoverArt::from_jpeg_or_seed(None, "Calyx").to_lines(),
    };
    let art_widget = Paragraph::new(art_lines)
        .block(Block::default().borders(Borders::ALL).title(" Portada "));
    frame.render_widget(art_widget, bottom_chunks[0]);

    let (track_title, track_artist_album) = match &app.current_track {
        Some(t) => (
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(truncate(&t.title, 48), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(truncate(&t.artist, 26), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" • ", Style::default().fg(Color::DarkGray)),
                Span::styled(truncate(&t.album, 26), Style::default().fg(Color::Gray)),
            ]),
        ),
        None => (
            Line::from(Span::styled(" Sin pista activa", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(" Selecciona una pista con Enter para reproducir", Style::default().fg(Color::DarkGray))),
        ),
    };

    let status_str = match app.playback_status() {
        calyx_core::models::PlaybackStatus::Playing => "▶ Reproduciendo",
        calyx_core::models::PlaybackStatus::Paused => "⏸ En Pausa",
        calyx_core::models::PlaybackStatus::Buffering => "⏳ Cargando...",
        calyx_core::models::PlaybackStatus::Stopped => "⏹ Detenido",
    };
    let state_vol_line = Line::from(vec![
        Span::styled(format!(" {status_str}"), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("   │   ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("Volumen: {:>3}%", (app.volume * 100.0) as u8), Style::default().fg(Color::Cyan)),
    ]);

    let (elapsed, total) = app.playback_progress();
    let elap_s = elapsed.as_secs();
    let tot_s = if total.as_secs() > 0 {
        total.as_secs()
    } else {
        app.current_track.as_ref().map(|t| (t.duration_ms / 1000) as u64).unwrap_or(0)
    };
    let rem_s = tot_s.saturating_sub(elap_s);
    let ratio = if tot_s > 0 { (elap_s as f64 / tot_s as f64).clamp(0.0, 1.0) } else { 0.0 };
    let bar_w = 28usize;
    let filled = (ratio * bar_w as f64) as usize;
    let bar_str: String = (0..bar_w).map(|i| if i < filled { '━' } else if i == filled { '╸' } else { '─' }).collect();
    let progress_line = Line::from(Span::styled(
        format!(
            " [{:02}:{:02} / {:02}:{:02} (-{:02}:{:02})] ┫{}┣ {:>3}%",
            elap_s / 60, elap_s % 60, tot_s / 60, tot_s % 60, rem_s / 60, rem_s % 60, bar_str, (ratio * 100.0) as u8
        ),
        Style::default().fg(Color::Yellow),
    ));

    let note_line = if !app.status_message.is_empty() {
        Line::from(Span::styled(format!(" {}", app.status_message), Style::default().fg(Color::Magenta)))
    } else {
        Line::from("")
    };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            track_title,
            track_artist_album,
            Line::from(""),
            state_vol_line,
            Line::from(""),
            progress_line,
            Line::from(""),
            note_line,
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Reproductor ")),
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