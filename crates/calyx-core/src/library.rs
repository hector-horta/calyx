use crate::error::{CalyxError, CalyxResult};
use crate::models::Track;
use reqwest::Client;
use serde_json::Value;

pub fn parse_library_songs_json(raw_json: &str, storefront: &str) -> CalyxResult<Vec<Track>> {
    let val: Value = serde_json::from_str(raw_json)
        .map_err(|e| CalyxError::Validation(format!("Invalid library JSON: {e}")))?;
    let items = val
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| CalyxError::Validation("No 'data' array found".into()))?;

    let mut tracks = Vec::with_capacity(items.len());
    for item in items {
        let id_str = item
            .pointer("/relationships/catalog/data/0/attributes/playParams/id")
            .or_else(|| item.pointer("/relationships/catalog/data/0/id"))
            .or_else(|| item.pointer("/attributes/playParams/catalogId"))
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
            })
            .or_else(|| {
                item.pointer("/attributes/playParams/purchasedId")
                    .and_then(|v| v.as_str().map(String::from))
            })
            .or_else(|| {
                item.pointer("/attributes/playParams/id")
                    .and_then(|v| v.as_str().map(String::from))
            })
            .or_else(|| item.get("id").and_then(|v| v.as_str().map(String::from)))
            .ok_or_else(|| CalyxError::Validation("Missing track ID".into()))?;

        let title = item
            .pointer("/attributes/name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Title");

        let artist = item
            .pointer("/attributes/artistName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Artist");

        let album = item
            .pointer("/attributes/albumName")
            .and_then(|v| v.as_str())
            .unwrap_or("Library");

        let duration_ms = item
            .pointer("/attributes/durationInMillis")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let artwork_url = item
            .pointer("/attributes/artwork/url")
            .and_then(|v| v.as_str())
            .map(|u| u.replace("{w}", "200").replace("{h}", "200"));

        let preview_url = item
            .pointer("/attributes/previews/0/url")
            .or_else(|| item.pointer("/relationships/catalog/data/0/attributes/previews/0/url"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let lib_id = item
            .pointer("/attributes/playParams/id")
            .or_else(|| item.get("id"))
            .and_then(|v| v.as_str())
            .filter(|s| s.starts_with("i."))
            .map(String::from);

        tracks.push(Track {
            id: id_str,
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            duration_ms,
            storefront: storefront.to_string(),
            preview_url,
            artwork_url,
            hls_manifest_url: lib_id,
            is_lossless: true,
            album_id: crate::album_id::album_id_from_song(item),
        });
    }
    Ok(tracks)
}

const API_ROOT: &str = "https://amp-api.music.apple.com";
const PAGE_SIZE: usize = 100;

/// Items on one page and the path of the next one. Only relative `/v1/`
/// paths are followed, so a response can never redirect us to another host.
fn page_info(body: &str) -> CalyxResult<(usize, Option<String>)> {
    let val: Value = serde_json::from_str(body)
        .map_err(|e| CalyxError::Validation(format!("Invalid Apple Music page JSON: {e}")))?;
    let count = val
        .get("data")
        .and_then(|d| d.as_array())
        .map_or(0, |a| a.len());
    let next = val
        .get("next")
        .and_then(|n| n.as_str())
        .filter(|n| n.starts_with("/v1/"))
        .map(str::to_string);
    Ok((count, next))
}

/// GETs `first_path` and every following page until `max_items` or the end.
async fn get_paged(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    first_path: String,
    max_items: usize,
    what: &str,
) -> CalyxResult<Vec<String>> {
    let mut bodies = Vec::new();
    let mut seen = 0;
    let mut next = Some(first_path);
    while let Some(path) = next.take() {
        let resp = http
            .get(format!("{API_ROOT}{path}"))
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(CalyxError::Auth(format!(
                "{what} HTTP status: {}",
                resp.status()
            )));
        }
        let body = resp.text().await?;
        let (count, next_path) = page_info(&body)?;
        seen += count;
        bodies.push(body);
        next = next_path.filter(|_| count > 0 && seen < max_items);
    }
    Ok(bodies)
}

pub(crate) fn require_session(dev_token: &str, user_token: &str, what: &str) -> CalyxResult<()> {
    if dev_token.is_empty() || user_token.is_empty() {
        return Err(CalyxError::Auth(format!(
            "Se requiere sesión activa ('u') para {what}"
        )));
    }
    Ok(())
}

/// Songs in the user's library, following pagination up to `limit`.
pub async fn fetch_user_library_songs(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    storefront: &str,
    limit: usize,
) -> CalyxResult<Vec<Track>> {
    require_session(dev_token, user_token, "sincronizar tu biblioteca")?;
    let first = format!("/v1/me/library/songs?include=catalog&limit={PAGE_SIZE}");
    let mut tracks = Vec::new();
    for body in get_paged(http, dev_token, user_token, first, limit, "Library").await? {
        tracks.extend(parse_library_songs_json(&body, storefront)?);
    }
    tracks.truncate(limit);
    Ok(tracks)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPlaylist {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The user may add songs to it (own playlists; not followed/shared ones).
    #[serde(default = "default_can_edit")]
    pub can_edit: bool,
}

fn default_can_edit() -> bool {
    true
}

pub fn parse_user_playlists_json(raw_json: &str) -> CalyxResult<Vec<UserPlaylist>> {
    let val: Value = serde_json::from_str(raw_json)
        .map_err(|e| CalyxError::Validation(format!("Invalid playlists JSON: {e}")))?;
    let items = val
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| CalyxError::Validation("No 'data' array in playlists".into()))?;
    let mut playlists = Vec::with_capacity(items.len());
    for item in items {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            let name = item
                .pointer("/attributes/name")
                .and_then(|v| v.as_str())
                .unwrap_or("Playlist");
            let desc = item
                .pointer("/attributes/description/standard")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            playlists.push(UserPlaylist {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                can_edit: item
                    .pointer("/attributes/canEdit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            });
        }
    }
    Ok(playlists)
}

/// The user's playlists, following pagination up to `limit`.
pub async fn fetch_user_playlists(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    limit: usize,
) -> CalyxResult<Vec<UserPlaylist>> {
    require_session(dev_token, user_token, "ver tus Playlists")?;
    let first = format!("/v1/me/library/playlists?limit={PAGE_SIZE}");
    let mut playlists = Vec::new();
    for body in get_paged(http, dev_token, user_token, first, limit, "Playlists").await? {
        playlists.extend(parse_user_playlists_json(&body)?);
    }
    playlists.truncate(limit);
    Ok(playlists)
}

/// Library playlist ids look like `p.AbC123`; anything else is rejected
/// before it is put into a URL path.
pub(crate) fn valid_playlist_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Tracks of a library playlist, following pagination up to `limit`.
pub async fn fetch_playlist_tracks(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    playlist_id: &str,
    storefront: &str,
    limit: usize,
) -> CalyxResult<Vec<Track>> {
    if !valid_playlist_id(playlist_id) {
        return Err(CalyxError::Validation(format!(
            "Invalid playlist id: {playlist_id:?}"
        )));
    }
    let first =
        format!("/v1/me/library/playlists/{playlist_id}/tracks?include=catalog&limit={PAGE_SIZE}");
    let mut tracks = Vec::new();
    for body in get_paged(http, dev_token, user_token, first, limit, "Playlist tracks").await? {
        tracks.extend(parse_library_songs_json(&body, storefront)?);
    }
    tracks.truncate(limit);
    Ok(tracks)
}

#[cfg(test)]
#[path = "library_tests.rs"]
mod tests;
