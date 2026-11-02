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
/// Personal Apple Music Library & Playlists synchronization.
pub mod library;
/// Interactive zero-friction Apple Music Sign-In.
pub mod login;
/// Synchronized and plain lyrics fetching & parsing.
pub mod lyrics;
/// Data models for tracks, playlists, and metadata.
pub mod models;
/// Playback queue management with repeat and shuffle support.
pub mod queue;
/// Pure-Rust Widevine L3 cryptographic primitives.
pub mod wv_crypto;
/// Built-in Android 9 L3 WVD device credentials.
pub mod wvd_builtin;

/// Re-exported public items from the library's modules.
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
pub use library::{
    fetch_playlist_tracks, fetch_user_library_songs, fetch_user_playlists,
    parse_library_songs_json, parse_user_playlists_json, UserPlaylist,
};
pub use login::LoginBrowserGuard;
pub use lyrics::{fetch_track_lyrics, parse_lrc, LyricLine};
pub use models::{DrmMode, PlaybackStatus, Track};
pub use queue::{PlaybackQueue, RepeatMode};
