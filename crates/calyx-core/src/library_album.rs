//! The album of a library song, as the user's library has it. The way in
//! when the song's catalog id is gone (Apple replaced the release): the
//! catalog answers 404 for it, but the library still knows its album.

use crate::error::{CalyxError, CalyxResult};
use crate::models::Track;
use crate::paging::{PAGE_SIZE, get_paged};
use reqwest::Client;
use serde_json::Value;

/// Most songs read from one library album.
const MAX_ALBUM_TRACKS: usize = 300;

/// Library album ids look like `l.AbC12`; anything else never reaches a URL.
fn valid_album_id(id: &str) -> bool {
    id.strip_prefix("l.")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// (album id, album name) from a `/v1/me/library/songs/{id}/albums` body.
fn album_ref(body: &str) -> Option<(String, String)> {
    let val: Value = serde_json::from_str(body).ok()?;
    let album = val.pointer("/data/0")?;
    let id = album.get("id")?.as_str().filter(|id| valid_album_id(id))?;
    let name = album
        .pointer("/attributes/name")
        .and_then(Value::as_str)
        .unwrap_or("Album");
    Some((id.to_string(), name.to_string()))
}

/// Album name and songs of the library album that holds `library_song_id`.
pub(crate) async fn fetch_library_album_of(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    library_song_id: &str,
    storefront: &str,
) -> CalyxResult<(String, Vec<Track>)> {
    crate::library::require_session(dev_token, user_token, "abrir el álbum")?;
    // Checked before it goes into the URL: only `i.` + alphanumerics pass.
    if crate::library_write::song_resource(library_song_id)? != "library-songs" {
        return Err(CalyxError::Validation(format!(
            "Invalid library song id: {library_song_id:?}"
        )));
    }

    let first = get_paged(
        http,
        dev_token,
        user_token,
        format!("/v1/me/library/songs/{library_song_id}/albums"),
        1,
        "Library album",
        None,
    )
    .await?;
    let (album_id, name) = first
        .first()
        .and_then(|body| album_ref(body))
        .ok_or_else(|| CalyxError::Validation("Album not found".into()))?;

    let path = format!("/v1/me/library/albums/{album_id}/tracks?include=catalog&limit={PAGE_SIZE}");
    let bodies = get_paged(
        http,
        dev_token,
        user_token,
        path,
        MAX_ALBUM_TRACKS,
        "Library album",
        None,
    )
    .await?;
    let mut tracks = Vec::new();
    for body in bodies {
        tracks.extend(crate::library::parse_library_songs_json(&body, storefront)?);
    }

    Ok((name, tracks))
}

#[cfg(test)]
#[path = "library_album_tests.rs"]
mod tests;
