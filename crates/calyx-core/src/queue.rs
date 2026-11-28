use crate::models::Track;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn next_mode(&self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlaybackQueue {
    upcoming: VecDeque<Track>,
    history: Vec<Track>,
    repeat_mode: RepeatMode,
    shuffle: bool,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_back(&mut self, track: Track) {
        self.upcoming.push_back(track);
    }

    pub fn enqueue_next(&mut self, track: Track) {
        self.upcoming.push_front(track);
    }

    pub fn remove(&mut self, index: usize) -> Option<Track> {
        self.upcoming.remove(index)
    }

    pub fn clear(&mut self) {
        self.upcoming.clear();
        self.history.clear();
    }

    /// Automatic advance (track finished): honours Repeat One.
    pub fn next_track(&mut self, current: Option<Track>) -> Option<Track> {
        self.advance(current, true)
    }

    /// Manual skip by the user: always moves on, even with Repeat One.
    pub fn skip_track(&mut self, current: Option<Track>) -> Option<Track> {
        self.advance(current, false)
    }

    fn advance(&mut self, current: Option<Track>, honor_repeat_one: bool) -> Option<Track> {
        if honor_repeat_one
            && self.repeat_mode == RepeatMode::One
            && let Some(c) = current
        {
            return Some(c);
        }

        if let Some(c) = current {
            self.history.push(c);
        }

        if self.upcoming.is_empty() {
            if self.repeat_mode == RepeatMode::All && !self.history.is_empty() {
                self.upcoming = self.history.drain(..).collect();
            } else {
                return None;
            }
        }

        if self.shuffle && self.upcoming.len() > 1 {
            self.upcoming.remove(random_index(self.upcoming.len()))
        } else {
            self.upcoming.pop_front()
        }
    }

    pub fn previous_track(&mut self, current: Option<Track>) -> Option<Track> {
        if self.history.is_empty() {
            return None;
        }
        if let Some(c) = current {
            self.upcoming.push_front(c);
        }
        self.history.pop()
    }

    pub fn toggle_repeat(&mut self) -> RepeatMode {
        self.repeat_mode = self.repeat_mode.next_mode();
        self.repeat_mode
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        self.shuffle = !self.shuffle;
        self.shuffle
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn upcoming(&self) -> &VecDeque<Track> {
        &self.upcoming
    }

    pub fn len(&self) -> usize {
        self.upcoming.len()
    }

    pub fn is_empty(&self) -> bool {
        self.upcoming.is_empty()
    }

    pub fn peek_next(&self) -> Option<&Track> {
        self.upcoming.front()
    }
}

/// A random index below `len` (0 for an empty list). Seeded by the standard
/// library's per-hasher random keys: the clock is not random enough (on
/// Windows its nanoseconds are always a multiple of 100).
fn random_index(len: usize) -> usize {
    use std::hash::{BuildHasher, Hasher};
    let random = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    (random % len.max(1) as u64) as usize
}

#[cfg(test)]
#[path = "../tests/unit/queue.rs"]
mod tests;
