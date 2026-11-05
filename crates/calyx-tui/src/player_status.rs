//! Text lines of the player panel (title, status, up next), adapted to its width.

use calyx_core::{models::PlaybackStatus, queue::RepeatMode};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use crate::app::TuiApp;
use crate::hints::KEY_STYLE;
use crate::player_spectrum::{NARROW_BELOW, WIDE_FROM};
use crate::ui::truncate;

const INFO: Style = Style::new().fg(Color::Rgb(120, 160, 160));
const TIME: Style = Style::new().fg(Color::Rgb(220, 225, 230)).add_modifier(Modifier::BOLD);
const OFF: Style = Style::new().fg(Color::DarkGray);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density { Narrow, Medium, Wide }

impl Density {
    pub fn for_width(inner: u16) -> Self {
        if inner < NARROW_BELOW { Self::Narrow } else if inner >= WIDE_FROM { Self::Wide } else { Self::Medium }
    }
}

/// Elapsed seconds, total seconds and played ratio (0..=1).
pub fn progress(app: &TuiApp) -> (u64, u64, f64) {
    let (elapsed, total) = app.playback_progress();
    let elap = elapsed.as_secs();
    let tot = total.as_secs().max(app.current_track.as_ref().map(|t| t.duration_ms / 1000).unwrap_or(0));
    let ratio = if tot > 0 { (elapsed.as_millis() as f64 / (tot as f64 * 1000.0)).clamp(0.0, 1.0) } else { 0.0 };
    (elap, tot, ratio)
}

pub fn header_line(app: &TuiApp, density: Density) -> Line<'static> {
    let Some(t) = &app.current_track else {
        return Line::from(Span::styled(app.language.no_active_playback(), OFF));
    };

    let mut spans = Vec::with_capacity(3);

    if density == Density::Narrow {
        let glyph = match app.playback_status() {
            PlaybackStatus::Playing => "▶ ",
            PlaybackStatus::Buffering => "↻ ",
            PlaybackStatus::Paused | PlaybackStatus::Stopped => "⏸ ",
        };
        spans.push(Span::styled(glyph, Style::new().fg(Color::Cyan)));
    }

    spans.push(Span::styled(
        format!("{} ", truncate(&t.title, 34)),
        Style::new()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    ));

    spans.push(Span::styled(
        format!("— {} • {}", truncate(&t.artist, 22), truncate(&t.album, 22)),
        Style::new().fg(Color::Cyan)
    ));

    Line::from(spans)
}

fn time_span(app: &TuiApp) -> Span<'static> {
    let (elapsed, total, _) = progress(app);
    let remaining = total.saturating_sub(elapsed);
    Span::styled(
        format!(
            "[{:02}:{:02} / {:02}:{:02} (-{:02}:{:02})]",
            elapsed / 60,
            elapsed % 60,
            total / 60,
            total % 60,
            remaining / 60,
            remaining % 60
        ),
        TIME
    )
}
fn switch_spans(app: &TuiApp, compact: bool) -> Vec<Span<'static>> {
    let l = app.language;
    let shuffle_style = if app.queue.is_shuffle() { Style::new().fg(Color::Rgb(200, 235, 245)) } else { OFF };
    let (rep_label, rep_icon, rep_style) = match app.queue.repeat_mode() {
        RepeatMode::Off => (l.btn_repeat(), "🔁", OFF),
        RepeatMode::All => (l.btn_repeat_all(), "🔁", Style::new().fg(Color::Cyan)),
        RepeatMode::One => (l.btn_repeat_one(), "🔂", Style::new().fg(Color::Cyan)),
    };
    let (shuf, rep) = if compact {
        ("🔀".to_string(), rep_icon.to_string())
    } else {
        (format!("🔀 {}", l.btn_shuffle()), format!("{rep_icon} {rep_label}"))
    };
    vec![
        Span::styled("s", KEY_STYLE),
        Span::raw(" "),
        Span::styled(shuf, shuffle_style),
        Span::styled("   │   ", OFF),
        Span::styled("r", KEY_STYLE),
        Span::raw(" "),
        Span::styled(rep, rep_style),
    ]
}

pub fn status_lines(app: &TuiApp, density: Density, width: u16) -> Vec<Line<'static>> {
    match density {
        Density::Wide => vec![spread(
            Line::from(time_span(app)),
            Line::from(switch_spans(app, false)),
            &app.status_message,
            width
        )],
        Density::Medium => vec![Line::from(time_span(app)), Line::from(switch_spans(app, false))],
        Density::Narrow => {
            let mut spans = vec![time_span(app), Span::raw("  ")];
            spans.extend(switch_spans(app, true));
            spans.push(Span::styled(
                format!("  Vol {}%", (app.volume * 100.0).round() as u8),
                OFF
            ));
            vec![Line::from(spans)]
        }
    }
}

fn spread(left: Line<'static>, center: Line<'static>, right: &str, width: u16) -> Line<'static> {
    let (w, lw, cw) = (width as usize, left.width(), center.width());
    let pad1 = (w.saturating_sub(cw) / 2).saturating_sub(lw).max(1);
    let room = w.saturating_sub(lw + pad1 + cw + 1);
    // Leave 1 spare column: wide glyphs (emoji) occupy 2 cells but count as 1 char.
    let right = Span::styled(truncate(right, room.saturating_sub(1)), INFO.add_modifier(Modifier::ITALIC));
    let pad2 = w.saturating_sub(lw + pad1 + cw + right.width()).max(1);
    let mut spans = left.spans;
    spans.push(Span::raw(" ".repeat(pad1)));
    spans.extend(center.spans);
    spans.push(Span::raw(" ".repeat(pad2)));
    spans.push(right);
    Line::from(spans)
}

pub fn footer_line(app: &TuiApp, density: Density, width: u16) -> Line<'static> {
    let l = app.language;
    let status = app.status_message.as_str();
    let next = app.queue.peek_next().map(|n| format!(
        "{}: {} — {} ({} {})",
        l.lbl_up_next(),
        truncate(&n.title, 34),
        truncate(&n.artist, 20),
        app.queue.len(),
        l.lbl_in_queue()
    ));
    match density {
        Density::Wide => Line::from(Span::styled(next.unwrap_or_default(), INFO)),
        Density::Narrow => {
            let text = if status.is_empty() { next.unwrap_or_default() } else { status.to_string() };
            Line::from(Span::styled(truncate(&text, width as usize), INFO))
        }
        Density::Medium => match (next, status.is_empty()) {
            (Some(n), true) => Line::from(Span::styled(n, INFO)),
            (Some(n), false) => Line::from(vec![
                Span::styled(format!("{n}  │  "), INFO),
                Span::styled(truncate(status, 42), INFO.add_modifier(Modifier::ITALIC)),
            ]),
            (None, false) => Line::from(Span::styled(status.to_string(), INFO)),
            (None, true) => Line::from(""),
        },
    }
}