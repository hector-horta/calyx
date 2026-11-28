//! The version in the top box title (" Calyx v0.1.13 "), plus the newer
//! release when the background check found one.

use crate::update_install::UpdateState;
use calyx_core::Language;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::sync::OnceLock;

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Tag of a newer release, set once by the background check.
static NEWER: OnceLock<String> = OnceLock::new();

/// Asks GitHub (on every start, see `calyx_core::update_check`) on a
/// tokio task; the title shows the answer whenever it arrives.
pub fn start_check() {
    tokio::spawn(async {
        if let Some(tag) = calyx_core::update_check::newer_release(VERSION).await {
            let _ = NEWER.set(tag);
        }
    });
}

/// Tag of the newer release, once the background check has found one.
pub fn newer() -> Option<&'static str> {
    NEWER.get().map(String::as_str)
}

/// " Calyx v0.1.13 ", or with a newer release " Calyx v0.1.13 → v0.1.14
/// disponible [V] ", where each step of the update replaces "disponible".
pub fn title(lang: Language, update: UpdateState) -> Line<'static> {
    title_with(lang, newer(), update)
}

fn title_with(lang: Language, newer: Option<&str>, update: UpdateState) -> Line<'static> {
    let dim = Style::new().fg(Color::DarkGray);
    let mut spans = vec![
        Span::raw(" Calyx "),
        Span::styled(format!("v{VERSION} "), dim),
    ];
    if let Some(tag) = newer {
        let (state, style, key) = crate::update_install::badge(lang, update);
        let tag_style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        spans.push(Span::styled(format!("→ {tag} "), tag_style));
        spans.push(Span::styled(format!(" {state} "), style));
        if key {
            spans.push(crate::hints::key_span("V", crate::hints::KEY_STYLE));
        }
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

/// Opens the newer release's page in the browser (does nothing until the
/// check has found one): 'V' where calyx cannot update itself, or after a
/// failed install.
pub fn open_release(app: &mut crate::app::TuiApp) {
    let Some(url) = NEWER
        .get()
        .and_then(|tag| calyx_core::update_check::release_url(tag))
    else {
        return;
    };
    if let Err(e) = calyx_core::login_firefox::open_url_in_system_browser(&url) {
        app.zones.player.set_status(e.to_string());
    }
}

#[cfg(test)]
#[path = "../tests/unit/version_badge.rs"]
mod tests;
