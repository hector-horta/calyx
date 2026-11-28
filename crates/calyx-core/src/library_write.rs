//! Writes to the user's Apple Music library (needs a logged-in session).

use crate::album_id::is_catalog_album_id;
use crate::error::{CalyxError, CalyxResult};
use reqwest::{Client, StatusCode};

pub(crate) const LIBRARY_URL: &str = "https://amp-api.music.apple.com/v1/me/library";

/// Target of `POST /v1/me/library?ids[albums]=<id>`; the id is validated
/// before it is put into the URL.
fn add_album_url(album_id: &str) -> CalyxResult<String> {
    if !is_catalog_album_id(album_id) {
        return Err(CalyxError::Validation(format!(
            "Invalid album id: {album_id:?}"
        )));
    }
    Ok(format!("{LIBRARY_URL}?ids[albums]={album_id}"))
}

/// Apple answers 202 Accepted, also when the album was already there.
pub(crate) fn check_status(status: StatusCode) -> CalyxResult<()> {
    if status.is_success() {
        return Ok(());
    }
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(CalyxError::Auth(format!(
            "Sesión Apple expirada (HTTP {status}). Presiona 'u' para login."
        )));
    }
    Err(CalyxError::Validation(format!(
        "Apple library HTTP {status}"
    )))
}

/// Resource type Apple expects for a song id: catalog ids are numeric ("songs"),
/// library-only ids look like `i.AbC` ("library-songs"); anything else is rejected.
pub(crate) fn song_resource(song_id: &str) -> CalyxResult<&'static str> {
    let library_id = song_id
        .strip_prefix("i.")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric()));
    match (is_catalog_album_id(song_id), library_id) {
        (true, _) => Ok("songs"),
        (_, true) => Ok("library-songs"),
        _ => Err(CalyxError::Validation(format!(
            "Invalid song id: {song_id:?}"
        ))),
    }
}

/// URL and body of `POST /v1/me/library/playlists/{id}/tracks` for one song.
/// Catalog ids are numeric ("songs"); library-only ids look like `i.AbC` ("library-songs").
fn add_to_playlist_request(
    playlist_id: &str,
    song_id: &str,
) -> CalyxResult<(String, serde_json::Value)> {
    if !crate::library::valid_playlist_id(playlist_id) {
        return Err(CalyxError::Validation(format!(
            "Invalid playlist id: {playlist_id:?}"
        )));
    }
    let kind = song_resource(song_id)?;
    let url = format!("{LIBRARY_URL}/playlists/{playlist_id}/tracks");
    let body = serde_json::json!({ "data": [{ "id": song_id, "type": kind }] });
    Ok((url, body))
}

/// Appends one song to a library playlist the user owns (Apple answers 204).
pub(crate) async fn add_track_to_playlist(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    playlist_id: &str,
    song_id: &str,
) -> CalyxResult<()> {
    crate::library::require_session(dev_token, user_token, "agregar a una playlist")?;
    let (url, body) = add_to_playlist_request(playlist_id, song_id)?;
    let resp = http
        .post(url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .json(&body)
        .send()
        .await?;
    check_status(resp.status())
}

/// Adds a catalog album to the user's library.
pub(crate) async fn add_album_to_library(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    album_id: &str,
) -> CalyxResult<()> {
    crate::library::require_session(dev_token, user_token, "agregar a tu biblioteca")?;
    let url = add_album_url(album_id)?;
    let resp = http
        .post(url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send()
        .await?;
    check_status(resp.status())
}

#[cfg(test)]
#[path = "../tests/unit/library_write.rs"]
mod tests;
