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
#[path = "../tests/unit/spinner.rs"]
mod tests;
