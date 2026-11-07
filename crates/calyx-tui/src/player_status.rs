//! Time and controls (shuffle, repeat, volume) lines of the player.

use crate::app::TuiApp;
use crate::hints::{KEY_STYLE, key_span};
use calyx_core::queue::RepeatMode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

const TIME: Style = Style::new()
    .fg(Color::Rgb(220, 225, 230))
    .add_modifier(Modifier::BOLD);
const OFF: Style = Style::new().fg(Color::DarkGray);
const SEP: &str = "  │  ";
/// Volume meter steps (block elements, not emoji).
const METER: [&str; 4] = ["▂", "▄", "▆", "█"];

/// Elapsed seconds, total seconds and played ratio (0..=1).
pub fn progress(app: &TuiApp) -> (u64, u64, f64) {
    let (elapsed, total) = app.playback_progress();
    let elap = elapsed.as_secs();
    let tot = total.as_secs().max(
        app.current_track
            .as_ref()
            .map(|t| t.duration_ms / 1000)
            .unwrap_or(0),
    );
    let ratio = if tot > 0 {
        (elapsed.as_millis() as f64 / (tot as f64 * 1000.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (elap, tot, ratio)
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
        TIME,
    )
}
/// "[s] Aleatorio │ [r] Repetir │ [- / +] Vol 85% ▂▄▆█": the playback controls
/// on one line. ASCII labels; the state shows as colour (and "todo"/"una").
/// `compact` (narrow boxes): shorter separators and no meter glyphs.
fn control_spans(app: &TuiApp, compact: bool) -> Vec<Span<'static>> {
    let sep = if compact { " │ " } else { SEP };
    let l = app.language;
    let on = Style::new().fg(Color::Cyan);
    let shuffle = if app.queue.is_shuffle() { on } else { OFF };
    let (repeat, repeat_style) = match app.queue.repeat_mode() {
        RepeatMode::Off => (l.btn_repeat(), OFF),
        RepeatMode::All => (l.btn_repeat_all(), on),
        RepeatMode::One => (l.btn_repeat_one(), on),
    };
    let vol = (app.volume * 100.0).round() as u8;
    let lit = ((vol as f64 / 100.0) * 4.0).round() as usize;
    let mut spans = vec![
        key_span("s", KEY_STYLE),
        Span::styled(format!(" {}", l.btn_shuffle()), shuffle),
        Span::styled(sep, OFF),
        key_span("r", KEY_STYLE),
        Span::styled(format!(" {repeat}"), repeat_style),
        Span::styled(sep, OFF),
        key_span("- / +", KEY_STYLE),
        Span::styled(format!(" Vol {vol}%"), TIME),
    ];
    if !compact {
        spans.push(Span::raw(" "));
        spans.extend((0..4).map(|i| Span::styled(METER[i], if i < lit { on } else { OFF })));
    }
    spans
}

/// Time on one line, the controls (shuffle, repeat, volume) on the next; both
/// centred on the same axis as the spectrum.
pub fn status_lines(app: &TuiApp, width: usize) -> Vec<Line<'static>> {
    let mut controls = Line::from(control_spans(app, false));
    if controls.width() > width {
        controls = Line::from(control_spans(app, true));
    }
    vec![Line::from(time_span(app)), controls]
}
