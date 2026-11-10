//! Core library for the Calyx music player.

/// Catalog album id of a song (relationships or song URL).
mod album_id;
/// Public API module.
pub mod api;
/// Audio processing and playback engine.
pub mod audio;
/// Local disk cache for decrypted streaming audio.
pub mod audio_cache;
/// Authentication and session management.
pub mod auth;
/// Pluggable Service Facade contract and native backend providers.
pub mod backend;
/// Multi-browser dynamic discovery (Chromium & Firefox/Gecko).
pub mod browser_discovery;
/// Tracks of the catalog album a song belongs to.
mod catalog_album;
/// Minimal Chrome DevTools Protocol transport.
pub mod cdp;
/// Database operations and catalog management.
pub mod db;
/// Off-thread M4A decoding to PCM.
pub mod decode;
/// DRM-related functionality (Widevine, Cenc, etc.).
pub mod drm;
/// Custom error types and utilities.
pub mod error;
/// The streaming facade contract (`StreamingBackend`).
mod facade;
/// MP4 file format processing.
pub mod fmp4;
/// HLS streaming support.
pub mod hls;
/// Internationalization and localized text resources.
pub mod i18n;
/// Personal Apple Music Library & Playlists synchronization.
pub mod library;
/// Writes to the user's library (add album).
mod library_write;
/// Interactive zero-friction Apple Music Sign-In.
pub mod login;
/// Gecko/Firefox family cookie extraction and login support.
pub mod login_firefox;
/// WebKit/Safari family binary cookies parsing and login support.
pub mod login_webkit;
/// Synchronized and plain lyrics fetching & parsing.
pub mod lyrics;
/// Native Apple Music TTML lyrics parsing and fetching.
pub mod lyrics_ttml;
/// Data models for tracks, playlists, and metadata.
pub mod models;
/// Paginated GETs on the user-scoped API.
mod paging;
/// Per-platform locations for session, history and caches.
pub mod paths;
/// Create a playlist with one song in it.
mod playlist_create;
/// Remove songs from, rename or delete a library playlist (web API).
mod playlist_edit;
/// Progress of paginated loads, shared with the UI.
pub mod progress;
/// Playback queue management with repeat and shuffle support.
pub mod queue;
/// Catalog searches behind the curated sections.
mod sections;
/// Precomputed visualizer frames.
pub mod spectrum;
/// Cache → full stream → preview strategy behind the facade.
mod stream_resolver;
/// Phonetic transliteration engine (Romaji, Hangul RR).
pub mod transliteration;
/// Whether a newer calyx release exists on GitHub (checked once a day).
pub mod update_check;
/// Full-track playback (webPlayback, Widevine license, HLS download).
mod webplay;
/// Pure-Rust Widevine L3 cryptographic primitives.
pub mod wv_crypto;
/// Built-in Android 9 L3 WVD device credentials.
pub mod wvd_builtin;

pub use api::AppleMusicClient;
pub use audio::NativeAudioEngine;
pub use audio_cache::AudioCache;
pub use auth::{SessionConfig, fetch_apple_developer_token};
pub use backend::{AppleMusicBackend, AudioSource, ResolvedAudio, SectionType, StreamingBackend};
pub use db::CatalogStore;
/// Re-exported public items from the library's modules.
pub use decode::{DecodedTrack, decode_track};
pub use drm::{CbcsDecryptor, PsshBox, WidevineDeviceL3};
pub use error::{CalyxError, CalyxResult};
pub use fmp4::{
    decrypt_apple_fmp4_stream, decrypt_fmp4_segment_in_place, scan_mp4_boxes,
    strip_enca_protection_atoms,
};
pub use hls::HlsStreamPlan;
pub use i18n::Language;
pub use library::{
    UserPlaylist, fetch_playlist_tracks, fetch_user_library_songs, fetch_user_playlists,
    parse_library_songs_json, parse_user_playlists_json,
};
pub use login::LoginBrowserGuard;
pub use lyrics::{LyricLine, fetch_track_lyrics, parse_lrc};
pub use lyrics_ttml::{fetch_apple_music_lyrics, parse_ttml, parse_ttml_time};
pub use models::{DrmMode, PlaybackStatus, Track};
pub use progress::LoadProgress;
pub use queue::{PlaybackQueue, RepeatMode};
pub use spectrum::{SPECTRUM_BANDS, SpectrumFrame};
pub use transliteration::transliterate;
