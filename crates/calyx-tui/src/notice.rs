//! Notices that must not get lost: an action waiting for a key, an error or a
//! result. They live apart from routine progress messages and are drawn in
//! reverse video with an ASCII tag, so a later "Loading..." never hides them.

use crate::messages::Msg;
use crate::messages_picker::PickerMsg;
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
                let left = self.tone.ttl().saturating_sub(self.at.elapsed()).as_secs() + 1;
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

impl Msg<'_> {
    /// Returns which notices must stand out (None = routine progress/info).
    pub fn tone(&self) -> Option<Tone> {
        match self {
            Msg::Enqueued { .. }
            | Msg::EnqueuedNext { .. }
            | Msg::RemovedFromQueue(..)
            | Msg::RemovedFromRecent(..)
            | Msg::QueueCleared
            | Msg::RecentCleared
            | Msg::ListEnqueued(..)
            | Msg::AlbumAddedToLibrary(..)
            | Msg::LoginOk => Some(Tone::Result),
            Msg::ListError { .. }
            | Msg::PlaylistsError(..)
            | Msg::PlaylistError { .. }
            | Msg::PlaylistPlayFailed(..)
            | Msg::AlbumError(..)
            | Msg::AlbumAddError { .. }
            | Msg::LoginError(..)
            | Msg::AudioError(..) => Some(Tone::Error),
            Msg::Picker(p) => p.tone(),
            _ => None,
        }
    }
}

impl PickerMsg<'_> {
    /// Returns the tone for this picker message.
    pub fn tone(&self) -> Option<Tone> {
        match self {
            PickerMsg::ConfirmRemove { .. } => Some(Tone::Action),
            PickerMsg::PlaylistCreated { .. }
            | PickerMsg::AddedToPlaylist { .. }
            | PickerMsg::Removed { .. } => Some(Tone::Result),
            PickerMsg::PlaylistCreateError { .. }
            | PickerMsg::AddToPlaylistError { .. }
            | PickerMsg::RemoveError { .. }
            | PickerMsg::NotEditable(..)
            | PickerMsg::NoLibraryId(..) => Some(Tone::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "notice_tests.rs"]
mod tests;
