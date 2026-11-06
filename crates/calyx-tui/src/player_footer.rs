//! Footer line of the player: "up next" and the status message, which is
//! only cut when it is wider than the whole panel.

use crate::ui::truncate;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

const INFO: Style = Style::new().fg(Color::Rgb(120, 160, 160));
const SEP: &str = "  │  ";

/// Columns `text` takes on screen (emoji count as 2).
fn cols(text: &str) -> usize {
    Span::raw(text).width()
}

/// Footer: "up next │ status" when both fit; otherwise the status (it has
/// priority over up next); otherwise up next.
pub fn footer(next: Option<String>, status: &str, width: usize) -> Line<'static> {
    let italic = INFO.add_modifier(Modifier::ITALIC);
    // One spare column: `truncate` counts chars, wide glyphs take 2 cells.
    let max = width.saturating_sub(1);
    match (next, status.is_empty()) {
        (Some(n), false) if cols(&n) + cols(SEP) + cols(status) <= width => Line::from(vec![
            Span::styled(format!("{n}{SEP}"), INFO),
            Span::styled(status.to_string(), italic),
        ]),
        (_, false) => Line::from(Span::styled(truncate(status, max), italic)),
        (Some(n), true) => Line::from(Span::styled(truncate(&n, max), INFO)),
        (None, true) => Line::from(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG: &str = "Playlists sincronizadas: 50 listas encontradas (Enter para abrir)";

    fn text(line: &Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn short_status_shares_line_with_up_next() {
        let line = footer(Some("A continuación: X".to_string()), "[AAC 256k]", 80);
        assert_eq!(text(&line), "A continuación: X  │  [AAC 256k]");
    }

    #[test]
    fn long_status_is_shown_whole_when_it_fits_alone() {
        let next = Some("A continuación: The Outsider — A Perfect Circle (22 en cola)".to_string());
        assert_eq!(text(&footer(next, LONG, 90)), LONG);
    }

    #[test]
    fn status_wider_than_panel_is_truncated() {
        let line = text(&footer(None, LONG, 40));
        assert!(line.chars().count() <= 39);
        assert!(line.ends_with('…'));
    }

    #[test]
    fn without_status_shows_up_next() {
        assert_eq!(
            text(&footer(Some("A continuación: X".to_string()), "", 80)),
            "A continuación: X"
        );
    }
}
