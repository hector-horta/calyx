//! What calyx is playing, as a small JSON file other apps can read
//! (`now_playing.json` in the data dir). It exists only while a song is
//! playing or paused; readers extrapolate the position from `updated_at_ms`
//! and ignore a file older than `STALE_AFTER` (calyx refreshes it sooner).

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Version of the JSON layout; bump it when a field changes meaning.
pub const SCHEMA: u32 = 1;
/// How often calyx rewrites the file while a song is loaded (heartbeat).
pub const REFRESH_EVERY: Duration = Duration::from_secs(15);
/// Readers drop a file older than this: calyx exited without removing it.
pub const STALE_AFTER: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayState {
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NowPlaying {
    pub schema: u32,
    pub player: &'static str,
    pub state: PlayState,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub artwork_url: Option<String>,
    pub duration_ms: u64,
    pub position_ms: u64,
    pub updated_at_ms: u64,
}

/// `now_playing.json` in calyx's data dir (`CALYX_HOME/data` when set).
pub fn now_playing_file() -> PathBuf {
    crate::paths::data_dir().join("now_playing.json")
}

/// Removes the file: nothing is playing (best effort, a missing file is fine).
pub fn clear(path: &Path) {
    if let Err(e) = std::fs::remove_file(path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "could not remove now_playing.json");
    }
}

/// Returns current Unix time in milliseconds since epoch, or zero if the system clock is invalid.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl NowPlaying {
    /// Writes the snapshot as pretty JSON to `path` atomically (temp file +
    /// rename), so a reader never sees a half-written file.
    pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
        let temp_path = path.with_extension("json.tmp");
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&temp_path, bytes)?;
        std::fs::rename(&temp_path, path)
    }
}

#[cfg(test)]
#[path = "now_playing_tests.rs"]
mod tests;
