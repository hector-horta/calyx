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
mod tests {
    use super::*;

    #[test]
    fn random_index_reaches_every_position() {
        // Lengths that divide 100 always gave 0 with the old clock-based pick.
        for len in [2usize, 4, 10] {
            let mut seen = vec![false; len];
            for _ in 0..2_000 {
                seen[random_index(len)] = true;
            }
            assert!(seen.iter().all(|s| *s), "len {len}: {seen:?}");
        }
        assert_eq!(random_index(0), 0);
    }

    fn dummy_track(id: &str) -> Track {
        Track {
            id: id.to_string(),
            title: format!("Title {id}"),
            artist: format!("Artist {id}"),
            album: format!("Album {id}"),
            duration_ms: 180_000,
            storefront: "us".to_string(),
            preview_url: None,
            artwork_url: None,
            hls_manifest_url: None,
            is_lossless: false,
            album_id: None,
        }
    }

    #[test]
    fn test_queue_ordering() {
        let mut q = PlaybackQueue::new();
        q.enqueue_back(dummy_track("1"));
        q.enqueue_back(dummy_track("2"));
        q.enqueue_next(dummy_track("0"));

        assert_eq!(q.len(), 3);
        assert_eq!(q.peek_next().unwrap().id, "0");
        assert_eq!(q.next_track(None).unwrap().id, "0");
        assert_eq!(q.next_track(None).unwrap().id, "1");
        assert_eq!(q.next_track(None).unwrap().id, "2");
        assert!(q.is_empty());
    }

    #[test]
    fn test_repeat_modes() {
        let mut q = PlaybackQueue::new();
        assert_eq!(q.repeat_mode(), RepeatMode::Off);
        q.toggle_repeat();
        assert_eq!(q.repeat_mode(), RepeatMode::All);
        q.toggle_repeat();
        assert_eq!(q.repeat_mode(), RepeatMode::One);

        let cur = dummy_track("current");
        let next = q.next_track(Some(cur.clone())).unwrap();
        assert_eq!(next.id, "current");
    }

    #[test]
    fn previous_without_history_does_not_duplicate() {
        let mut q = PlaybackQueue::new();
        q.enqueue_back(dummy_track("1"));
        assert!(q.previous_track(Some(dummy_track("cur"))).is_none());
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn previous_returns_last_played() {
        let mut q = PlaybackQueue::new();
        q.enqueue_back(dummy_track("1"));
        q.enqueue_back(dummy_track("2"));
        let t1 = q.next_track(None).unwrap();
        let t2 = q.next_track(Some(t1)).unwrap();
        assert_eq!(q.previous_track(Some(t2)).unwrap().id, "1");
        assert_eq!(q.peek_next().unwrap().id, "2");
    }

    #[test]
    fn skip_ignores_repeat_one() {
        let mut q = PlaybackQueue::new();
        q.toggle_repeat();
        q.toggle_repeat();
        q.enqueue_back(dummy_track("2"));
        assert_eq!(q.skip_track(Some(dummy_track("1"))).unwrap().id, "2");
    }

    #[test]
    fn test_repeat_all_cycle() {
        let mut q = PlaybackQueue::new();
        q.toggle_repeat(); // All
        q.enqueue_back(dummy_track("1"));

        let t1 = q.next_track(None).unwrap();
        assert!(q.is_empty());

        let t2 = q.next_track(Some(t1)).unwrap();
        assert_eq!(t2.id, "1");
    }
}
