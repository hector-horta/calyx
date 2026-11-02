use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::TuiApp;
use crate::ui::truncate;

pub fn render_player_panel(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(30)])
        .split(area);

    let art_lines = match &app.current_artwork {
        Some(a) => a.to_lines(),
        None => crate::artwork::TerminalCoverArt::from_jpeg_or_seed(None, "Calyx").to_lines(),
    };
    frame.render_widget(
        Paragraph::new(art_lines).block(Block::default().borders(Borders::ALL).title(" Portada ")),
        bottom_chunks[0],
    );

    // Player Controls
    let track_title = match &app.current_track {
        Some(t) => Line::from(Span::styled(
            format!(" {}", truncate(&t.title, 45)),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        )),
        None => Line::from(Span::styled(" Sin reproducción activa", Style::default().fg(Color::DarkGray))),
    };

    let track_artist_album = match &app.current_track {
        Some(t) => Line::from(Span::styled(
            format!(" {} • {}", truncate(&t.artist, 25), truncate(&t.album, 25)),
            Style::default().fg(Color::Cyan)
        )),
        None => Line::from(Span::styled(" Selecciona una pista con Enter o 'p'", Style::default().fg(Color::DarkGray))),
    };

    let status_str = match app.playback_status() {
        calyx_core::models::PlaybackStatus::Playing => "▶  Reproduciendo",
        calyx_core::models::PlaybackStatus::Paused => "⏸  En Pausa",
        calyx_core::models::PlaybackStatus::Buffering => "⏳  Cargando...",
        calyx_core::models::PlaybackStatus::Stopped => "⏹  Detenido",
    };

    let state_vol_line = Line::from(Span::styled(
        format!(
            " {status_str}   │   Vol: {:>3}%   │   {}   │   {}",
            (app.volume * 100.0) as u8,
            if app.queue.is_shuffle() { "🔀 ON" } else { "🔀 OFF" },
            app.queue.repeat_mode().badge()
        ),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    ));

    let (elapsed, total) = app.playback_progress();
    let elap_s = elapsed.as_secs();
    let tot_s = total.as_secs().max(app.current_track.as_ref().map(|t| t.duration_ms / 1000).unwrap_or(0));
    let rem_s = tot_s.saturating_sub(elap_s);
    let ratio = if tot_s > 0 { (elap_s as f64 / tot_s as f64).clamp(0.0, 1.0) } else { 0.0 };
    let filled = (ratio * 28.0) as usize;
    let bar_str: String = (0..28).map(|i| if i < filled { '━' } else if i == filled { '╸' } else { '─' }).collect();
    let progress_line = Line::from(Span::styled(
        format!(" [{:02}:{:02} / {:02}:{:02} (-{:02}:{:02})] ┫{bar_str}┣ {:>3}%", elap_s / 60, elap_s % 60, tot_s / 60, tot_s % 60, rem_s / 60, rem_s % 60, (ratio * 100.0) as u8),
        Style::default().fg(Color::Yellow),
    ));

let queue_cue = match app.queue.peek_next() {
    Some(next) => Line::from(Span::styled(
        format!(" A continuación: {} — {} ({} en cola)", truncate(&next.title, 26), truncate(&next.artist, 18), app.queue.len()),
        Style::default().fg(Color::Cyan),
    )),
    None => Line::from(Span::styled(" Cola vacía: [a - Encolar al final] ─── [n - Reproducir a continuación]", Style::default().fg(Color::Gray))),
};

let note_line = Line::from(Span::styled(
    format!(" {}", app.status_message),
    Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)
));

    let bottom_controls = Line::from(vec![
        Span::raw("─"),
        Span::styled("[Enter - Play]", Style::default().fg(Color::Cyan)),
        Span::raw(" ─── "),
        Span::styled("[p - Play Lista]", Style::default().fg(Color::Cyan)),
        Span::raw(" ─── "),
        Span::styled("[Espacio - Pausa]", Style::default().fg(Color::Cyan)),
        Span::raw(" ─── "),
        Span::styled("[< / > - Ant/Sig]", Style::default().fg(Color::Cyan)),
        Span::raw(" ─── "),
        Span::styled("[s - Shuffle]", Style::default().fg(if app.queue.is_shuffle() { Color::Yellow } else { Color::Cyan })),
        Span::raw(" ─── "),
        Span::styled("[r - Repeat]", Style::default().fg(Color::Cyan)),
        Span::raw(" ─── "),
        Span::styled("[-/+ - Vol]", Style::default().fg(Color::Cyan)),
        Span::raw("─"),
    ]);

    frame.render_widget(
        Paragraph::new(vec![Line::from(""), track_title, track_artist_album, Line::from(""), state_vol_line, progress_line, queue_cue, note_line])
            .block(Block::default().borders(Borders::ALL).title_top(" Reproductor ").title_bottom(bottom_controls)),
        bottom_chunks[1],
    );
}