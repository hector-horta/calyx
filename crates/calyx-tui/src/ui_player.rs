//! Player panel: cover art + responsive spectrum, status and up-next lines.

use calyx_core::models::PlaybackStatus;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::TuiApp;
use crate::hints::{hint_bar, LABEL_STYLE};
use crate::player_spectrum::{spectrum_row, BarLayout};
use crate::player_status::{footer_line, header_line, progress, status_lines, Density};

/// Bar colours from the top row down.
const ROW_COLORS: [Color; 3] = [Color::Rgb(255, 215, 0), Color::Rgb(0, 240, 180), Color::Rgb(0, 200, 100)];
const OFF: Style = Style::new().fg(Color::DarkGray);

pub fn render_player_panel(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(30)])
        .split(area);
    let art_lines = match &app.current_artwork {
        Some(a) => a.to_lines(),
        None => crate::artwork::TerminalCoverArt::from_jpeg_or_seed(None, "Calyx").to_lines(),
    };
    frame.render_widget(Paragraph::new(art_lines).block(Block::default().borders(Borders::ALL)), cols[0]);

    let l = app.language;
    let block = Block::default().borders(Borders::ALL).title_bottom(hint_bar(&[
        ("Enter", l.btn_play(), LABEL_STYLE),
        (l.key_space(), l.btn_pause(), LABEL_STYLE),
        ("< / >", l.btn_prev_next(), LABEL_STYLE),
        ("- / +", l.btn_volume(), LABEL_STYLE),
    ]));
    let inner_w = block.inner(cols[1]).width;
    let layout = BarLayout::for_width(inner_w);
    let density = Density::for_width(inner_w);
    let (_, _, ratio) = progress(app);
    let spec = app.current_spectrum();

    let mut lines = vec![header_line(app, density), Line::from("")];
    for (i, color) in ROW_COLORS.iter().enumerate() {
        let r = (2 - i) as u8;
        let mut row = Vec::new();
        if layout.show_sides {
            row.extend(icon_row(app, r));
            row.push(Span::raw("  "));
        }
        row.extend(spectrum_row(&spec, r, *color, layout, ratio));
        if layout.show_sides {
            row.push(Span::raw("  "));
            row.extend(volume_cell(app, r));
        }
        lines.push(Line::from(row));
    }
    lines.extend(status_lines(app, density, inner_w));
    lines.push(Line::from(""));
    lines.push(footer_line(app, density, inner_w));
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center).block(block), cols[1]);
}

/// One row of the big play/pause icon (6 columns).
fn icon_row(app: &TuiApp, r: u8) -> Vec<Span<'static>> {
    let row_idx = (2 - r) as usize;
    let blank = || vec![Span::raw("      ")];
    match app.playback_status() {
        PlaybackStatus::Playing => app.play_icon.rows.get(row_idx).cloned().unwrap_or_else(blank),
        PlaybackStatus::Paused | PlaybackStatus::Stopped => app.pause_icon.rows.get(row_idx).cloned().unwrap_or_else(blank),
        PlaybackStatus::Buffering => vec![Span::styled(if r == 1 { "  ↻   " } else { "      " }, Style::new().fg(Color::Cyan))],
    }
}

/// Volume meter column (6 columns): label, percentage, then the step bar.
fn volume_cell(app: &TuiApp, r: u8) -> Vec<Span<'static>> {
    let vol = (app.volume * 100.0).round() as u8;
    match r {
        2 => vec![Span::styled("   VOL", OFF)],
        1 => vec![Span::styled(format!("  {:>4}", format!("{vol}%")), Style::new().fg(Color::White).add_modifier(Modifier::BOLD))],
        _ => {
            let lit = ((vol as f64 / 100.0) * 4.0).round() as usize;
            let steps = ["▂", "▄", "▆", "█"].into_iter().enumerate().map(|(i, s)| Span::styled(s, if i < lit { Style::new().fg(Color::Cyan) } else { OFF }));
            std::iter::once(Span::raw("  ")).chain(steps).collect()
        }
    }
}