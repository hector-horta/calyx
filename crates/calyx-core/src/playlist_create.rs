//! Create a library playlist that already contains one song (a single request).

use crate::error::{CalyxError, CalyxResult};
use crate::library::UserPlaylist;
use crate::library_write::{LIBRARY_URL, check_status, song_resource};
use reqwest::Client;
use serde_json::Value;

/// Longest playlist name accepted, in characters.
const MAX_NAME_CHARS: usize = 100;

/// Validates and sanitizes a playlist name by trimming whitespace, checking for empty input,
/// control characters, and length constraints.
pub(crate) fn clean_playlist_name(name: &str) -> CalyxResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CalyxError::Validation("Playlist name is empty".to_string()));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(CalyxError::Validation(
            "Playlist name has control characters".to_string(),
        ));
    }
    if trimmed.chars().count() > MAX_NAME_CHARS {
        Err(CalyxError::Validation(format!(
            "Playlist name is longer than {MAX_NAME_CHARS} characters"
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

/// Creates a POST request to `/v1/me/library/playlists` for creating a playlist with the given song.
fn create_playlist_request(name: &str, song_id: &str) -> CalyxResult<(String, Value)> {
    let name = clean_playlist_name(name)?;
    let kind = song_resource(song_id)?;
    let url = format!("{LIBRARY_URL}/playlists");
    Ok((
        url,
        serde_json::json!({
            "attributes": { "name": name },
            "relationships": { "tracks": {
                "data": [{"id": song_id, "type": kind}]
            }}
        }),
    ))
}

/// Parses the JSON response from Apple's API for a newly created playlist and extracts its details.
fn parse_created_playlist(body: &str) -> CalyxResult<UserPlaylist> {
    let playlists = crate::library::parse_user_playlists_json(body)?;
    playlists
        .into_iter()
        .next()
        .ok_or_else(|| CalyxError::Validation("Apple did not return the new playlist".to_string()))
}

/// Creates a library playlist named `name` that already contains `song_id`.
pub(crate) async fn create_playlist_with_track(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    name: &str,
    song_id: &str,
) -> CalyxResult<UserPlaylist> {
    crate::library::require_session(dev_token, user_token, "crear una playlist")?;
    let (url, body) = create_playlist_request(name, song_id)?;
    let resp = http
        .post(url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .json(&body)
        .send()
        .await?;
    check_status(resp.status())?;
    parse_created_playlist(&resp.text().await?)
}

#[cfg(test)]
#[path = "playlist_create_tests.rs"]
mod tests;
