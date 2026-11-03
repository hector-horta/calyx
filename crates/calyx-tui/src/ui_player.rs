use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
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
        Paragraph::new(art_lines).block(Block::default().borders(Borders::ALL)),
        bottom_chunks[0],
    );

let header_line = match &app.current_track {
    Some(t) => Line::from(vec![
        Span::styled(format!("{} ", truncate(&t.title, 34)), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(format!("— {} • {}", truncate(&t.artist, 22), truncate(&t.album, 22)), Style::default().fg(Color::Cyan)),
    ]),
    None => Line::from(Span::styled(app.language.no_active_playback(), Style::default().fg(Color::DarkGray))),
};

let status_str = match app.playback_status() {
    calyx_core::models::PlaybackStatus::Playing => app.language.status_playing(),
    calyx_core::models::PlaybackStatus::Paused => app.language.status_paused(),
    calyx_core::models::PlaybackStatus::Buffering => app.language.status_buffering(),
    calyx_core::models::PlaybackStatus::Stopped => app.language.status_stopped(),
};

let (elapsed, total) = app.playback_progress();
let elap_s = elapsed.as_secs();
let tot_s = total.as_secs().max(app.current_track.as_ref().map(|t| t.duration_ms / 1000).unwrap_or(0));
let rem_s = tot_s.saturating_sub(elap_s);
let ratio = if tot_s > 0 { (elapsed.as_millis() as f64 / (tot_s as f64 * 1000.0)).clamp(0.0, 1.0) } else { 0.0 };
let (shuf_str, shuf_col) = if app.queue.is_shuffle() { ("🔀  ON [s]", Color::Yellow) } else { ("🔀  OFF [s]", Color::Green) };
let (rep_str, rep_col) = match app.queue.repeat_mode() {
    calyx_core::queue::RepeatMode::Off => ("🔁  OFF [r]", Color::Green),
    calyx_core::queue::RepeatMode::All => ("🔁  ALL [r]", Color::Yellow),
    calyx_core::queue::RepeatMode::One => ("🔂  ONE [r]", Color::Yellow),
};
let time_line = Line::from(Span::styled(
    format!("[{:02}:{:02} / {:02}:{:02} (-{:02}:{:02})]", elap_s / 60, elap_s % 60, tot_s / 60, tot_s % 60, rem_s / 60, rem_s % 60),
    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
));

let state_line = Line::from(vec![
    Span::styled(status_str.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    Span::styled(format!("  │  Vol: {:>3}%  │  ", (app.volume * 100.0) as u8), Style::default().fg(Color::Green)),
    Span::styled(shuf_str, Style::default().fg(shuf_col)),
    Span::styled("  │  ", Style::default().fg(Color::Green)),
    Span::styled(rep_str, Style::default().fg(rep_col)),
]);

let spec = app.current_spectrum();
let lit = ((ratio * 32.0).round() as usize).clamp(0, 32);
let srow = |r: u8, c: Color| -> Line<'static> {
    let mut v = Vec::new();
    for (b, &val) in spec.iter().enumerate() {
        let idx = if val == 0 && r == 0 { 1 } else { val.saturating_sub(r * 8).min(8) as usize };
        let col = if val == 0 { Color::Rgb(45, 60, 60) } else if b < lit { c } else { Color::Rgb(55, 80, 80) };
        v.push(Span::styled(format!("{} ", [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'][idx]), Style::default().fg(col)));
    }
    Line::from(v)
};

let (lbl_next, lbl_queue) = match app.language {
    calyx_core::Language::Es => ("A continuación", "en cola"),
    calyx_core::Language::En => ("Up next", "in queue"),
};

let footer_line = match (app.queue.peek_next(), app.status_message.is_empty()) {
    (Some(next), true) => Line::from(Span::styled(format!("{lbl_next}: {} — {} ({} {lbl_queue})", truncate(&next.title, 28), truncate(&next.artist, 20), app.queue.len()), Style::default().fg(Color::Cyan))),
    (Some(next), false) => Line::from(vec![
        Span::styled(format!("{lbl_next}: {} — {} ({})  │  ", truncate(&next.title, 22), truncate(&next.artist, 14), app.queue.len()), Style::default().fg(Color::Cyan)),
        Span::styled(truncate(&app.status_message, 42), Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
    ]),
    (None, false) => Line::from(Span::styled(format!("{}", app.status_message), Style::default().fg(Color::White).add_modifier(Modifier::ITALIC))),
    (None, true) => Line::from(""),
};

    let bottom_controls = Line::from(vec![
        Span::raw("─"),
        Span::styled("[Enter - Play]", Style::default().fg(Color::Cyan)),
        Span::raw(" ── "),
        Span::styled(app.language.btn_pause(), Style::default().fg(Color::Cyan)),
        Span::raw(" ── "),
        Span::styled(app.language.btn_prev_next(), Style::default().fg(Color::Cyan)),
        Span::raw(" ── "),
        Span::styled("[-/+ - Vol]", Style::default().fg(Color::Cyan)),
        Span::raw("─"),
    ]);

    frame.render_widget(
        Paragraph::new(vec![
            header_line,
            Line::from(""),
            srow(2, Color::Rgb(255, 215, 0)),
            srow(1, Color::Rgb(0, 240, 180)),
            srow(0, Color::Rgb(0, 200, 100)),
            time_line,
            state_line,
            Line::from(""),
            footer_line,
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title_bottom(bottom_controls)),
        bottom_chunks[1],
    );
}