//! Tracks of a catalog album, so a song can open the album it belongs to.

use crate::album_id::{album_id_from_song, is_catalog_album_id};
use crate::api::AppleMusicClient;
use crate::error::{CalyxError, CalyxResult};
use crate::models::Track;
use serde_json::{Value, json};

const CATALOG_ROOT: &str = "https://amp-api.music.apple.com/v1/catalog";

/// Album name and its songs from a `/v1/catalog/{sf}/albums/{id}?include=tracks` body.
/// Music videos and other non-song items are skipped.
pub(crate) fn parse_album_json(val: &Value, storefront: &str) -> CalyxResult<(String, Vec<Track>)> {
    let album = val
        .pointer("/data/0")
        .ok_or_else(|| CalyxError::Validation("Album not found".into()))?;
    let name = album
        .pointer("/attributes/name")
        .and_then(Value::as_str)
        .unwrap_or("Album")
        .to_string();
    let songs: Vec<Value> = album
        .pointer("/relationships/tracks/data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|t| t.get("type").and_then(Value::as_str) == Some("songs"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let tracks = crate::library::parse_library_songs_json(
        &json!({ "data": songs }).to_string(),
        storefront,
    )?;
    Ok((name, tracks))
}

/// GET on the public catalog (developer token only; works without login).
async fn get_catalog(client: &AppleMusicClient, path: &str, what: &str) -> CalyxResult<Value> {
    let dev = client
        .developer_token
        .as_deref()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| CalyxError::Auth("Falta Developer Token".to_string()))?;
    let resp = client
        .http
        .get(format!("{CATALOG_ROOT}/{}/{path}", client.storefront))
        .header("Authorization", format!("Bearer {dev}"))
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send()
        .await?;
    client.parse_apple_json(resp, what).await
}

/// The track's album id, asking the catalog when the list did not carry it
/// (only possible when the track id is a catalog song id).
pub(crate) async fn resolve_album_id(
    client: &AppleMusicClient,
    track: &Track,
) -> CalyxResult<String> {
    if let Some(id) = track
        .album_id
        .as_deref()
        .filter(|id| is_catalog_album_id(id))
    {
        return Ok(id.to_string());
    }
    if !is_catalog_album_id(&track.id) {
        return Err(CalyxError::Validation(format!(
            "Sin álbum de catálogo para '{}'",
            track.title
        )));
    }
    let val = get_catalog(
        client,
        &format!("songs/{}?include=albums", track.id),
        "song",
    )
    .await?;
    val.pointer("/data/0")
        .and_then(album_id_from_song)
        .ok_or_else(|| CalyxError::Validation(format!("Sin álbum para '{}'", track.title)))
}

/// Album name and songs of the album `track` belongs to.
pub(crate) async fn fetch_album_of(
    client: &AppleMusicClient,
    track: &Track,
) -> CalyxResult<(String, Vec<Track>)> {
    let album_id = resolve_album_id(client, track).await?;
    let val = get_catalog(
        client,
        &format!("albums/{album_id}?include=tracks"),
        "album",
    )
    .await?;
    parse_album_json(&val, &client.storefront)
}

#[cfg(test)]
#[path = "../tests/unit/catalog_album.rs"]
mod tests;
