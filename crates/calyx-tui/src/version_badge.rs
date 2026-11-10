//! The version in the top box title (" Calyx v0.1.13 "), plus the newer
//! release when the background check found one.

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

/// " Calyx v0.1.13 " or " Calyx v0.1.13 → v0.1.14 disponible ".
pub fn title(lang: Language) -> Line<'static> {
    title_with(lang, NEWER.get().map(String::as_str))
}

fn title_with(lang: Language, newer: Option<&str>) -> Line<'static> {
    let dim = Style::new().fg(Color::DarkGray);
    let mut spans = vec![
        Span::raw(" Calyx "),
        Span::styled(format!("v{VERSION} "), dim),
    ];
    if let Some(tag) = newer {
        let available = match lang {
            Language::Es => "disponible",
            Language::En => "available",
        };
        let style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        spans.push(Span::styled(format!("→ {tag} {available} "), style));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn shows_the_version_and_a_newer_release_when_known() {
        let plain = text(&title_with(Language::Es, None));
        assert_eq!(plain, format!(" Calyx v{VERSION} "));
        let es = text(&title_with(Language::Es, Some("v9.0.0")));
        assert!(es.ends_with("→ v9.0.0 disponible "));
        let en = text(&title_with(Language::En, Some("v9.0.0")));
        assert!(en.ends_with("→ v9.0.0 available "));
    }
}
