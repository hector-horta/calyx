//! Notices that must not get lost: an action waiting for a key, an error or a
//! result. They live apart from routine progress messages and are drawn in
//! reverse video with an ASCII tag, so a later "Loading..." never hides them.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Waiting for a key (e.g. "press d again"): yellow, with a countdown.
    Action,
    Error,
    Result,
}

impl Tone {
    pub fn tag(self) -> &'static str {
        match self {
            Tone::Action => "[!]",
            Tone::Error => "[x]",
            Tone::Result => "[ok]",
        }
    }

    pub fn ttl(self) -> Duration {
        match self {
            Tone::Action => Duration::from_secs(4), // same window as the 'd' confirmation
            Tone::Error => Duration::from_secs(6),
            Tone::Result => Duration::from_secs(3),
        }
    }

    pub fn style(self) -> Style {
        let (bg, fg) = match self {
            Tone::Action => (Color::Yellow, Color::Black),
            Tone::Error => (Color::Red, Color::White),
            Tone::Result => (Color::Green, Color::Black),
        };
        Style::new().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
    }
}

pub struct Notice {
    pub text: String,
    pub tone: Tone,
    pub at: Instant,
}

impl Notice {
    pub fn new(text: String, tone: Tone) -> Self {
        Self {
            text,
            tone,
            at: Instant::now(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.at.elapsed() < self.tone.ttl()
    }

    /// Returns a Line representing the notice's text in reverse video style, padded to `width` columns.
    pub fn line(&self, width: usize) -> Line<'static> {
        // The countdown goes right after the tag so a cut text never hides it.
        let countdown = match self.tone {
            Tone::Action => {
                // Rounded up: 3.2 s left shows "4s", and a fresh notice "4s", not "5s".
                let left_ms = self
                    .tone
                    .ttl()
                    .saturating_sub(self.at.elapsed())
                    .as_millis();
                let left = left_ms.div_ceil(1000);
                format!(" ({left}s)")
            }
            _ => String::new(),
        };
        let text = format!(" {}{countdown} {} ", self.tone.tag(), self.text);
        let mut text: String = text.chars().take(width).collect();
        let len = text.chars().count(); // chars, not bytes: "¿Quitar…" must pad right
        if len < width {
            text.push_str(&" ".repeat(width - len));
        }
        Line::from(Span::styled(text, self.tone.style()))
    }
}

#[cfg(test)]
#[path = "notice_tests.rs"]
mod tests;
