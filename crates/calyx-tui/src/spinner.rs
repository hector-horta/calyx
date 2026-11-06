//! "Rotating dots" spinner marking the track whose audio is being cached.
//! The UI redraws every ~50 ms, so the frame is derived from elapsed time.

use std::time::Instant;

const FRAMES: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
const FRAME_MS: u128 = 100;
const BLANK: &str = " ";

/// Track being prefetched into the audio cache and when that started.
pub type Caching = Option<(String, Instant)>;

fn frame_at(elapsed_ms: u128) -> &'static str {
    FRAMES[(elapsed_ms / FRAME_MS) as usize % FRAMES.len()]
}

/// Current frame of a spinner started at `since` (also used by the status line).
pub fn frame_since(since: Instant) -> &'static str {
    frame_at(since.elapsed().as_millis())
}

/// One-column cell: the spinner when `track_id` is being cached, blank otherwise.
pub fn cache_cell(caching: &Caching, track_id: &str) -> &'static str {
    match caching {
        Some((id, started)) if id == track_id => frame_at(started.elapsed().as_millis()),
        _ => BLANK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_advance_and_wrap() {
        assert_eq!(frame_at(0), "⣾");
        assert_eq!(frame_at(99), "⣾");
        assert_eq!(frame_at(100), "⣽");
        assert_eq!(frame_at(800), "⣾");
    }

    #[test]
    fn only_the_cached_track_spins() {
        let caching: Caching = Some(("42".to_string(), Instant::now()));
        assert!(FRAMES.contains(&cache_cell(&caching, "42")));
        assert_eq!(cache_cell(&caching, "7"), BLANK);
        assert_eq!(cache_cell(&None, "42"), BLANK);
    }
}
