//! Remove songs from a library playlist through the music.apple.com web API
//! (amp-api). Apple's public API only allows adding, so this is the web
//! player's own request, verified on a real account on 2026-11-07.
//!
//! Also verified that day (not wired yet), for future features:
//! - rename: `PATCH {LIBRARY_URL}/playlists/{id}` + `{"attributes":{"name":..}}` -> 204
//! - delete playlist: `DELETE {LIBRARY_URL}/playlists/{id}` -> 204

use crate::error::{CalyxError, CalyxResult};
use crate::library::valid_playlist_id;
use crate::library_write::LIBRARY_URL;
use reqwest::{Client, Method};
use serde_json::Value;

/// Checks if the given ID matches Apple's library song format.
fn valid_library_song_id(id: &str) -> bool {
    id.strip_prefix("i.").is_some_and(|s| {
        !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric())
    })
}

/// URL of the DELETE that removes library songs from a library playlist, as the
/// web player does it (verified 2026-11-07: 204 and the song is gone). `mode=all`
/// is required (without it: 400 "No mode supplied") and removes every copy of
/// the song in the playlist. A JSON body instead of query ids is rejected.
pub(crate) fn remove_tracks_url(playlist_id: &str, song_ids: &[&str]) -> CalyxResult<String> {
    if !valid_playlist_id(playlist_id) {
        return Err(CalyxError::Validation(format!(
            "Invalid playlist id: {playlist_id:?}"
        )));
    }
    if song_ids.is_empty() || song_ids.iter().any(|id| !valid_library_song_id(id)) {
        return Err(CalyxError::Validation(format!(
            "Invalid library song ids: {:?}",
            song_ids
        )));
    }

    Ok(format!(
        "{LIBRARY_URL}/playlists/{playlist_id}/tracks?ids[library-songs]={}&mode=all",
        song_ids.join(",")
    ))
}

/// Sends an HTTP request to Apple's music API with the provided credentials and body.
pub(crate) async fn send(
    http: &Client,
    method: Method,
    url: &str,
    body: Option<&Value>,
    dev_token: &str,
    user_token: &str,
) -> CalyxResult<(u16, String)> {
    crate::library::require_session(dev_token, user_token, "editar una playlist")?;
    let mut req = http
        .request(method, url)
        .header("Authorization", format!("Bearer {}", dev_token))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/");
    if let Some(b) = body {
        req = req.json(b);
    }
    let resp = req.send().await?;
    let status = resp.status().as_u16();
    let text = resp.text().await?;
    Ok((status, text))
}

/// Removes a single library song from a playlist using the Apple Music API.
pub(crate) async fn remove_from_playlist(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    playlist_id: &str,
    song_id: &str,
) -> CalyxResult<()> {
    let url = remove_tracks_url(playlist_id, &[song_id])?;
    let (status, body) = send(http, Method::DELETE, &url, None, dev_token, user_token).await?;
    match status {
        200..=299 => Ok(()),
        401 | 403 => Err(CalyxError::Auth(format!(
            "Sesión Apple expirada (HTTP {status}). Presiona 'u' para login."
        ))),
        _ => Err(CalyxError::Validation(format!(
            "Apple rechazó la operación (HTTP {status}): {}",
            body.chars().take(160).collect::<String>()
        ))),
    }
}

#[cfg(test)]
#[path = "playlist_edit_tests.rs"]
mod tests;
