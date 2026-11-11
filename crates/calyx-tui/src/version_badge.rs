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

/// Asks GitHub (at most once a day, see `calyx_core::update_check`) on a
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

/// " Calyx v0.1.13 ", " Calyx v0.1.13 → v0.1.14 disponible [V] ", and while
/// `update` runs, "… → v0.1.14 descargando…" / "… instalada, reinicia".
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
        let busy = matches!(update, UpdateState::Installing | UpdateState::Installed);
        let state = match (lang, update) {
            (Language::Es, UpdateState::Installing) => "descargando…",
            (Language::En, UpdateState::Installing) => "downloading…",
            (Language::Es, UpdateState::Installed) => "instalada, reinicia",
            (Language::En, UpdateState::Installed) => "installed, restart",
            (Language::Es, _) => "disponible",
            (Language::En, _) => "available",
        };
        let style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        spans.push(Span::styled(format!("→ {tag} {state} "), style));
        if !busy {
            spans.push(crate::hints::key_span("V", crate::hints::KEY_STYLE));
            spans.push(Span::raw(" "));
        }
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
        app.status_message = e.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn shows_the_version_and_a_newer_release_when_known() {
        let plain = text(&title_with(Language::Es, None, UpdateState::Idle));
        assert_eq!(plain, format!(" Calyx v{VERSION} "));
        let es = text(&title_with(Language::Es, Some("v9.0.0"), UpdateState::Idle));
        assert!(es.ends_with("→ v9.0.0 disponible [V] "));
        let en = text(&title_with(Language::En, Some("v9.0.0"), UpdateState::Idle));
        assert!(en.ends_with("→ v9.0.0 available [V] "));
        let busy = text(&title_with(
            Language::Es,
            Some("v9.0.0"),
            UpdateState::Installing,
        ));
        assert!(busy.ends_with("→ v9.0.0 descargando… "));
        let done = text(&title_with(
            Language::Es,
            Some("v9.0.0"),
            UpdateState::Installed,
        ));
        assert!(done.ends_with("→ v9.0.0 instalada, reinicia "));
    }
}
