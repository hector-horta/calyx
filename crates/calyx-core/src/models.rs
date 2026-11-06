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
mod tests {
    use super::*;

    fn t(title: &str, artist: &str) -> Track {
        Track {
            id: "1".to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: "Album".to_string(),
            duration_ms: 1_000,
            storefront: "us".to_string(),
            preview_url: None,
            artwork_url: None,
            hls_manifest_url: None,
            is_lossless: false,
        }
    }

    #[test]
    fn same_song_ignores_suffixes() {
        assert!(
            t("Digital Love (Remastered)", "Daft Punk")
                .is_same_song(&t("Digital Love", "Daft Punk"))
        );
    }

    #[test]
    fn different_title_is_not_same() {
        assert!(!t("One More Time", "Daft Punk").is_same_song(&t("Digital Love", "Daft Punk")));
    }

    #[test]
    fn featured_artist_still_matches() {
        assert!(
            t("Get Lucky", "Daft Punk")
                .is_same_song(&t("Get Lucky", "Daft Punk, Pharrell Williams"))
        );
    }

    #[test]
    fn different_artist_is_not_same() {
        assert!(!t("Hurt", "Nine Inch Nails").is_same_song(&t("Hurt", "Johnny Cash")));
    }

    #[test]
    fn non_ascii_titles_do_not_panic() {
        assert!(t("夜に駆ける - Remix", "YOASOBI").is_same_song(&t("夜に駆ける", "YOASOBI")));
        assert!(t("Beyoncé (Live)", "Beyoncé").is_same_song(&t("Beyoncé", "Beyoncé")));
    }
}
