//! The streaming facade contract: everything the UI needs from a music service.

use crate::backend::{ResolvedAudio, SectionType};
use crate::error::CalyxResult;
use crate::library::UserPlaylist;
use crate::lyrics::LyricLine;
use crate::models::Track;

pub trait StreamingBackend: Send + Sync + 'static {
    /// Human-readable service name.
    fn service_name(&self) -> &'static str;
    /// Whether a user session is available.
    fn is_logged_in(&self) -> bool;
    /// Catalog search.
    fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Curated section (home, new releases, radio).
    fn fetch_section(
        &self,
        section: SectionType,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Songs in the user's library.
    fn fetch_user_library(
        &self,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// The user's playlists.
    fn fetch_user_playlists(
        &self,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<UserPlaylist>>> + Send;
    /// Tracks of one playlist.
    fn fetch_playlist_tracks(
        &self,
        playlist_id: &str,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    /// Name and songs of the album `track` belongs to.
    fn fetch_album_of(
        &self,
        track: &Track,
    ) -> impl std::future::Future<Output = CalyxResult<(String, Vec<Track>)>> + Send;
    /// Adds the album `track` belongs to to the user's library (needs login).
    fn add_album_to_library(
        &self,
        track: &Track,
    ) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
    /// Fills missing preview/artwork URLs from the catalog.
    fn complete_track(&self, track: Track) -> impl std::future::Future<Output = Track> + Send;
    /// Playable audio: cache, then full stream, then preview.
    fn resolve_audio(
        &self,
        track: &Track,
    ) -> impl std::future::Future<Output = CalyxResult<ResolvedAudio>> + Send;
    /// Caches the full stream ahead of time (best effort).
    fn prefetch_audio(&self, track: &Track) -> impl std::future::Future<Output = ()> + Send;
    /// Raw cover-art bytes.
    fn fetch_artwork(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = CalyxResult<Vec<u8>>> + Send;
    /// Synced lyrics; empty when none exist.
    fn fetch_lyrics(
        &self,
        track: &Track,
    ) -> impl std::future::Future<Output = Vec<LyricLine>> + Send;
    /// Interactive login; persists the new session.
    fn login(&self) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
    /// Fetches the public developer token if the session has none.
    fn ensure_developer_token(&self) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
}
