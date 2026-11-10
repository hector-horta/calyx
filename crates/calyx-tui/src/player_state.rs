//! One-line playback state shown above the artist in the song box.

use crate::app::TuiApp;
use calyx_core::{Language, models::PlaybackStatus};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// "► REPRODUCIENDO", "▌▌ EN PAUSA", the spinner while it loads, "■ DETENIDO".
/// Text glyphs, not emoji, so every terminal draws them the same way.
pub fn state_line(app: &TuiApp) -> Line<'static> {
    let es = app.language == Language::Es;
    let (mark, text, color) = match app.playback_status() {
        PlaybackStatus::Playing => (
            "►",
            if es { "REPRODUCIENDO" } else { "PLAYING" },
            Color::Cyan,
        ),
        PlaybackStatus::Paused => ("▌▌", if es { "EN PAUSA" } else { "PAUSED" }, Color::Yellow),
        PlaybackStatus::Buffering => (
            app.activity.spinner().unwrap_or("⣾"),
            if es { "CARGANDO" } else { "LOADING" },
            Color::Cyan,
        ),
        PlaybackStatus::Stopped => (
            "■",
            if es { "DETENIDO" } else { "STOPPED" },
            Color::DarkGray,
        ),
    };
    let style = Style::new().fg(color).add_modifier(Modifier::BOLD);
    Line::from(Span::styled(format!("{mark} {text}"), style))
}
