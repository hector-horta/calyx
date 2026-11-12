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
mod tests {
    use super::*;
    use std::time::Instant;

    fn text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn shows_the_version_and_each_update_step() {
        let plain = text(&title_with(Language::Es, None, UpdateState::Idle));
        assert_eq!(plain, format!(" Calyx v{VERSION} "));
        let tag = Some("v9.0.0");
        let es = text(&title_with(Language::Es, tag, UpdateState::Idle));
        assert!(es.ends_with("→ v9.0.0  disponible [V] "), "{es}");
        let en = text(&title_with(Language::En, tag, UpdateState::Idle));
        assert!(en.ends_with("→ v9.0.0  available [V] "), "{en}");
        let asked = text(&title_with(
            Language::Es,
            tag,
            UpdateState::Asked(Instant::now()),
        ));
        assert!(
            asked.contains("presiona V otra vez para instalar (4s)"),
            "{asked}"
        );
        let now = Instant::now();
        let busy = text(&title_with(Language::Es, tag, UpdateState::Installing(now)));
        assert!(
            busy.contains("descargando…") && !busy.contains("[V]"),
            "{busy}"
        );
        let done = text(&title_with(Language::Es, tag, UpdateState::Installed));
        assert!(done.contains("instalada: reinicia calyx"), "{done}");
        let failed = text(&title_with(Language::En, tag, UpdateState::Failed(now)));
        assert!(failed.contains("could not install"), "{failed}");
    }

    #[test]
    fn an_old_question_or_failure_goes_back_to_available() {
        let long_ago = Instant::now() - std::time::Duration::from_secs(60);
        for state in [UpdateState::Asked(long_ago), UpdateState::Failed(long_ago)] {
            let t = text(&title_with(Language::Es, Some("v9.0.0"), state));
            assert!(t.ends_with(" disponible [V] "), "{t}");
        }
    }
}
