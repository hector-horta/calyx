use crate::error::{CalyxError, CalyxResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub storefront: String,
    pub preview_url: Option<String>,
    pub artwork_url: Option<String>,
    pub hls_manifest_url: Option<String>,
    pub is_lossless: bool,
    /// Catalog album id, when known (opens the album the song belongs to).
    #[serde(default)]
    pub album_id: Option<String>,
}

impl Track {
    pub fn validate(&self) -> CalyxResult<()> {
        if self.id.trim().is_empty() || self.title.trim().is_empty() {
            return Err(CalyxError::Validation(
                "Track ID and title must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn formatted_duration(&self) -> String {
        let total_secs = self.duration_ms / 1000;
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// The song's id in the user's library (`i.…`), when it came from there.
    /// It keeps working when Apple drops the catalog id the library points
    /// at (a replaced release): playback already relies on it.
    pub fn library_id(&self) -> Option<&str> {
        self.hls_manifest_url
            .as_deref()
            .filter(|id| id.starts_with("i."))
    }

    /// True when `other` is very likely the same song: same normalized title,
    /// and one normalized artist contains the other (covers "feat." credits).
    pub fn is_same_song(&self, other: &Track) -> bool {
        let (title_a, title_b) = (normalize(&self.title), normalize(&other.title));
        if title_a.is_empty() || title_a != title_b {
            return false;
        }
        let (artist_a, artist_b) = (normalize(&self.artist), normalize(&other.artist));
        artist_a.contains(&artist_b) || artist_b.contains(&artist_a)
    }
}

/// Lowercase, cut at the first "(", "[" or " - " (remaster/feat. suffixes),
/// keep only letters, digits and spaces, and collapse runs of spaces.
fn normalize(s: &str) -> String {
    let lower = s.to_lowercase();
    let end = [lower.find('('), lower.find('['), lower.find(" - ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(lower.len());
    let kept: String = lower[..end]
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrmMode {
    FreePreview,
    WidevineL3Cenc,
    WidevineL3Cbcs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

#[cfg(test)]
#[path = "../tests/unit/models.rs"]
mod tests;
