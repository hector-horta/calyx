//! Streaming facade: everything the UI needs from a music service.
//!
//! The TUI talks only to [`StreamingBackend`]; tokens, HTTP clients, the
//! audio cache and the stream-resolution strategy stay behind it.

use std::sync::RwLock;

use crate::api::AppleMusicClient;
use crate::audio_cache::AudioCache;
use crate::auth::SessionConfig;
use crate::error::CalyxResult;
use crate::library::UserPlaylist;
use crate::lyrics::LyricLine;
use crate::models::Track;
use crate::stream_resolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Home,
    NewReleases,
    Radio,
}

/// Where a playable buffer came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSource {
    Cache,
    FullStream,
    Preview,
}

/// Encoded audio ready to decode; `note` explains why a lower tier was used.
#[derive(Debug, Clone)]
pub struct ResolvedAudio {
    pub bytes: Vec<u8>,
    pub source: AudioSource,
    pub note: Option<String>,
}

pub trait StreamingBackend: Send + Sync + 'static {
    /// Human-readable service name.
    fn service_name(&self) -> &'static str;
    /// Whether a user session is available.
    fn is_logged_in(&self) -> bool;
    /// Catalog search.
    fn search(&self, query: &str, limit: usize) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Curated section (home, new releases, radio).
    fn fetch_section(&self, section: SectionType) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Songs in the user's library.
    fn fetch_user_library(&self) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// The user's playlists.
    fn fetch_user_playlists(&self) -> impl std::future::Future<Output = CalyxResult<Vec<UserPlaylist>>> + Send;
    /// Tracks of one playlist.
    fn fetch_playlist_tracks(&self, playlist_id: &str) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Fills missing preview/artwork URLs from the catalog.
    fn complete_track(&self, track: Track) -> impl std::future::Future<Output = Track> + Send;
    /// Playable audio: cache, then full stream, then preview.
    fn resolve_audio(&self, track: &Track) -> impl std::future::Future<Output = CalyxResult<ResolvedAudio>> + Send;
    /// Caches the full stream ahead of time (best effort).
    fn prefetch_audio(&self, track: &Track) -> impl std::future::Future<Output = ()> + Send;
    /// Raw cover-art bytes.
    fn fetch_artwork(&self, url: &str) -> impl std::future::Future<Output = CalyxResult<Vec<u8>>> + Send;
    /// Synced lyrics; empty when none exist.
    fn fetch_lyrics(&self, track: &Track) -> impl std::future::Future<Output = Vec<LyricLine>> + Send;
    /// Interactive login; persists the new session.
    fn login(&self) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
    /// Fetches the public developer token if the session has none.
    fn ensure_developer_token(&self) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
}

/// Apple Music implementation of the facade. Session and client live behind
/// locks so the backend can be shared (`Arc`) by background tasks; every
/// method works on cheap snapshots and never holds a lock across `.await`.
pub struct AppleMusicBackend {
    client: RwLock<AppleMusicClient>,
    session: RwLock<SessionConfig>,
    cache: AudioCache,
}

impl AppleMusicBackend {
    pub fn new(session: SessionConfig, cache: AudioCache) -> CalyxResult<Self> {
        let mut client = AppleMusicClient::new(&session.storefront)?;
        client.set_tokens(session.developer_token.clone(), session.media_user_token.clone());
        Ok(Self { client: RwLock::new(client), session: RwLock::new(session), cache })
    }

    fn client(&self) -> AppleMusicClient {
        self.client.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn session(&self) -> SessionConfig {
        self.session.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Applies `f` to the session and persists it.
    fn update_session(&self, f: impl FnOnce(&mut SessionConfig)) -> CalyxResult<()> {
        let mut session = self.session.write().unwrap_or_else(|e| e.into_inner());
        f(&mut session);
        session.save()
    }

    /// Persists the UI language in the session file.
    pub fn set_language(&self, language: crate::i18n::Language) -> CalyxResult<()> {
        self.update_session(|s| s.language = Some(language))
    }

    /// (developer token, user token, storefront) for the user-scoped endpoints.
    fn user_ctx(&self) -> (String, String, String) {
        let s = self.session();
        (s.developer_token.unwrap_or_default(), s.media_user_token.unwrap_or_default(), s.storefront)
    }
}

/// "top hits <current year>" without pulling in a date crate.
fn new_releases_query() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("top hits {}", 1970 + secs / 31_556_952)
}

impl StreamingBackend for AppleMusicBackend {
    fn service_name(&self) -> &'static str {
        "Apple Music"
    }

    fn is_logged_in(&self) -> bool {
        self.session().is_logged_in()
    }

    async fn search(&self, query: &str, limit: usize) -> CalyxResult<Vec<Track>> {
        self.client().search_catalog(query, limit).await
    }

    async fn fetch_section(&self, section: SectionType) -> CalyxResult<Vec<Track>> {
        let query = match section {
            SectionType::Home => "top hits".to_string(),
            SectionType::NewReleases => new_releases_query(),
            SectionType::Radio => "electronic chill radio mix".to_string(),
        };
        self.client().search_catalog(&query, 25).await
    }

    async fn fetch_user_library(&self) -> CalyxResult<Vec<Track>> {
        let (dev, usr, sf) = self.user_ctx();
        crate::library::fetch_user_library_songs(self.client().http(), &dev, &usr, &sf, 50).await
    }

    async fn fetch_user_playlists(&self) -> CalyxResult<Vec<UserPlaylist>> {
        let (dev, usr, _) = self.user_ctx();
        crate::library::fetch_user_playlists(self.client().http(), &dev, &usr, 50).await
    }

    async fn fetch_playlist_tracks(&self, playlist_id: &str) -> CalyxResult<Vec<Track>> {
        let (dev, usr, sf) = self.user_ctx();
        crate::library::fetch_playlist_tracks(self.client().http(), &dev, &usr, playlist_id, &sf).await
    }

    async fn complete_track(&self, track: Track) -> Track {
        stream_resolver::complete_track(&self.client(), track).await
    }

    async fn resolve_audio(&self, track: &Track) -> CalyxResult<ResolvedAudio> {
        stream_resolver::resolve_audio(&self.client(), &self.session(), &self.cache, track).await
    }

    async fn prefetch_audio(&self, track: &Track) {
        stream_resolver::prefetch(&self.client(), &self.session(), &self.cache, track).await
    }

    async fn fetch_artwork(&self, url: &str) -> CalyxResult<Vec<u8>> {
        self.client().fetch_stream_bytes(url).await
    }

    async fn fetch_lyrics(&self, track: &Track) -> Vec<LyricLine> {
        let (dev, usr, sf) = self.user_ctx();
        let apple_ctx = (!dev.is_empty()).then_some((dev.as_str(), usr.as_str(), sf.as_str(), track.id.as_str()));
        crate::lyrics::fetch_track_lyrics(self.client().http(), &track.title, &track.artist, apple_ctx).await
    }

    async fn login(&self) -> CalyxResult<()> {
        let mut client = self.client();
        let (dev, user, storefront, guard) = client.login_and_verify_user().await?;
        self.update_session(|s| {
            s.developer_token = Some(dev);
            s.media_user_token = Some(user);
            s.storefront = storefront;
        })?;
        *self.client.write().unwrap_or_else(|e| e.into_inner()) = client;
        if let Some(guard) = guard {
            guard.kill_after_save().await;
        }
        Ok(())
    }

    async fn ensure_developer_token(&self) -> CalyxResult<()> {
        if self.session().developer_token.is_some_and(|t| !t.is_empty()) {
            return Ok(());
        }
        let mut client = self.client();
        let token = client.sync_developer_token().await?;
        self.update_session(|s| s.developer_token = Some(token))?;
        *self.client.write().unwrap_or_else(|e| e.into_inner()) = client;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> AppleMusicBackend {
        let session = SessionConfig { storefront: "us".to_string(), ..Default::default() };
        AppleMusicBackend::new(session, AudioCache::with_dir(std::env::temp_dir().join("calyx_backend_test"), 1024)).unwrap()
    }

    #[test]
    fn test_apple_music_backend_metadata() {
        let backend = backend();
        assert_eq!(backend.service_name(), "Apple Music");
        assert!(!backend.is_logged_in());
    }

    #[test]
    fn new_releases_query_uses_a_plausible_year() {
        let q = new_releases_query();
        let year: u64 = q.trim_start_matches("top hits ").parse().unwrap();
        assert!((2024..2100).contains(&year), "{q}");
    }

    #[test]
    fn backend_is_shareable_across_tasks() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<AppleMusicBackend>();
    }
}
