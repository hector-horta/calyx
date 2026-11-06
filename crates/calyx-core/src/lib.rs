//! Core library for the Calyx music player.

/// Public API module.
pub mod api;
/// Audio processing and playback engine.
pub mod audio;
/// Authentication and session management.
pub mod auth;
/// Minimal Chrome DevTools Protocol transport.
pub mod cdp;
/// Database operations and catalog management.
pub mod db;
/// DRM-related functionality (Widevine, Cenc, etc.).
pub mod drm;
/// Custom error types and utilities.
pub mod error;
/// MP4 file format processing.
pub mod fmp4;
/// HLS streaming support.
pub mod hls;
/// Internationalization and localized text resources.
pub mod i18n;
/// Personal Apple Music Library & Playlists synchronization.
pub mod library;
/// Interactive zero-friction Apple Music Sign-In.
pub mod login;
/// Multi-browser dynamic discovery (Chromium & Firefox/Gecko).
pub mod browser_discovery;
/// Gecko/Firefox family cookie extraction and login support.
pub mod login_firefox;
/// WebKit/Safari family binary cookies parsing and login support.
pub mod login_webkit;
/// Synchronized and plain lyrics fetching & parsing.
pub mod lyrics;
/// Phonetic transliteration engine (Romaji, Hangul RR).
pub mod transliteration;
/// Native Apple Music TTML lyrics parsing and fetching.
pub mod lyrics_ttml;
/// Data models for tracks, playlists, and metadata.
pub mod models;
/// Playback queue management with repeat and shuffle support.
pub mod queue;
/// Local disk cache for decrypted streaming audio.
pub mod audio_cache;
/// Pure-Rust Widevine L3 cryptographic primitives.
pub mod wv_crypto;
/// Built-in Android 9 L3 WVD device credentials.
pub mod wvd_builtin;
/// Pluggable Service Facade contract and native backend providers.
pub mod backend;
/// Cache → full stream → preview strategy behind the facade.
mod stream_resolver;
/// Per-platform locations for session, history and caches.
pub mod paths;
/// Off-thread M4A decoding to PCM.
pub mod decode;
/// Precomputed visualizer frames.
pub mod spectrum;

/// Re-exported public items from the library's modules.
pub use decode::{decode_track, DecodedTrack};
pub use spectrum::{SpectrumFrame, SPECTRUM_BANDS};
pub use backend::{AppleMusicBackend, AudioSource, ResolvedAudio, SectionType, StreamingBackend};
pub use api::AppleMusicClient;
pub use audio::NativeAudioEngine;
pub use auth::{fetch_apple_developer_token, SessionConfig};
pub use db::CatalogStore;
pub use drm::{CbcsDecryptor, PsshBox, WidevineDeviceL3};
pub use error::{CalyxError, CalyxResult};
pub use fmp4::{
    decrypt_apple_fmp4_stream, decrypt_fmp4_segment_in_place, scan_mp4_boxes,
    strip_enca_protection_atoms,
};
pub use hls::HlsStreamPlan;
pub use i18n::Language;
pub use library::{
    fetch_playlist_tracks, fetch_user_library_songs, fetch_user_playlists,
    parse_library_songs_json, parse_user_playlists_json, UserPlaylist,
};
pub use login::LoginBrowserGuard;
pub use lyrics::{fetch_track_lyrics, parse_lrc, LyricLine};
pub use transliteration::transliterate;
pub use lyrics_ttml::{fetch_apple_music_lyrics, parse_ttml, parse_ttml_time};
pub use models::{DrmMode, PlaybackStatus, Track};
pub use queue::{PlaybackQueue, RepeatMode};
pub use audio_cache::AudioCache;
