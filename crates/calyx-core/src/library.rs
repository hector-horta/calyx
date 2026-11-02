use reqwest::Client;
use serde_json::Value;
use crate::error::{CalyxError, CalyxResult};
use crate::models::Track;

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
            .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_u64().map(|n| n.to_string())))
            .or_else(|| item.pointer("/attributes/playParams/purchasedId").and_then(|v| v.as_str().map(String::from)))
            .or_else(|| item.pointer("/attributes/playParams/id").and_then(|v| v.as_str().map(String::from)))
            .or_else(|| item.get("id").and_then(|v| v.as_str().map(String::from)))
            .ok_or_else(|| CalyxError::Validation("Missing track ID".into()))?;

        let title = item.pointer("/attributes/name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Title");

        let artist = item.pointer("/attributes/artistName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Artist");

        let album = item.pointer("/attributes/albumName")
            .and_then(|v| v.as_str())
            .unwrap_or("Library");

        let duration_ms = item.pointer("/attributes/durationInMillis")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let artwork_url = item.pointer("/attributes/artwork/url")
            .and_then(|v| v.as_str())
            .map(|u| u.replace("{w}", "200").replace("{h}", "200"));

        let preview_url = item.pointer("/attributes/previews/0/url")
            .or_else(|| item.pointer("/relationships/catalog/data/0/attributes/previews/0/url"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let lib_id = item.pointer("/attributes/playParams/id")
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
            is_lossless: true
        });
    }
    Ok(tracks)
}

pub async fn fetch_user_library_songs(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    storefront: &str,
    limit: usize
) -> CalyxResult<Vec<Track>> {
    if dev_token.is_empty() || user_token.is_empty() {
        return Err(CalyxError::Auth("Se requiere sesión activa ('u') para sincronizar tu biblioteca".into()));
    }

    let url = format!(
        "https://amp-api.music.apple.com/v1/me/library/songs?include=catalog&limit={}",
        limit.clamp(1, 100)
    );

    let resp = http
        .get(&url)
        .header("Authorization", format!("Bearer {}", dev_token))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(CalyxError::Auth(format!("Library HTTP status: {}", resp.status())));
    }
    parse_library_songs_json(&resp.text().await?, storefront)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPlaylist {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub fn parse_user_playlists_json(raw_json: &str) -> CalyxResult<Vec<UserPlaylist>> {
    let val: Value = serde_json::from_str(raw_json)
        .map_err(|e| CalyxError::Validation(format!("Invalid playlists JSON: {e}")))?;
    let items = val.get("data").and_then(|d| d.as_array())
        .ok_or_else(|| CalyxError::Validation("No 'data' array in playlists".into()))?;
    let mut playlists = Vec::with_capacity(items.len());
    for item in items {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            let name = item.pointer("/attributes/name").and_then(|v| v.as_str()).unwrap_or("Playlist");
            let desc = item.pointer("/attributes/description/standard").and_then(|v| v.as_str()).unwrap_or("");
            playlists.push(UserPlaylist { id: id.to_string(), name: name.to_string(), description: desc.to_string() });
        }
    }
    Ok(playlists)
}

pub async fn fetch_user_playlists(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    limit: usize,
) -> CalyxResult<Vec<UserPlaylist>> {
    if dev_token.is_empty() || user_token.is_empty() {
        return Err(CalyxError::Auth("Se requiere sesión activa ('u') para ver tus Playlists".into()));
    }
    let url = format!("https://amp-api.music.apple.com/v1/me/library/playlists?limit={}", limit.clamp(1, 100));
    let resp = http.get(&url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send().await?;
    if !resp.status().is_success() {
        return Err(CalyxError::Auth(format!("Playlists HTTP status: {}", resp.status())));
    }
    parse_user_playlists_json(&resp.text().await?)
}

pub async fn fetch_playlist_tracks(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    playlist_id: &str,
    storefront: &str,
) -> CalyxResult<Vec<Track>> {
    let url = format!("https://amp-api.music.apple.com/v1/me/library/playlists/{playlist_id}/tracks?include=catalog");
    let resp = http.get(&url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send().await?;
    if !resp.status().is_success() {
        return Err(CalyxError::Auth(format!("Playlist tracks HTTP status: {}", resp.status())));
    }
    parse_library_songs_json(&resp.text().await?, storefront)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_library_songs_json() {
        let sample_json = r#"
            {
                "data": [
                    {
                        "id": "i.12345",
                        "attributes": {
                            "name": "Song Title",
                            "artistName": "Artist Name",
                            "albumName": "Album Name",
                            "durationInMillis": 180000,
                            "artwork": { "url": "https://example.com/artwork/{w}x{h}.jpg" },
                            "playParams": {
                                "catalogId": "1169918082",
                                "purchasedId": null
                            }
                        }
                    }
                ]
            }
        "#;

        let result = parse_library_songs_json(sample_json, "uy").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "1169918082");
        assert_eq!(result[0].title, "Song Title");
        assert_eq!(result[0].artwork_url.as_deref(), Some("https://example.com/artwork/200x200.jpg"));
    }

    #[test]
    fn test_parse_user_playlists_json() {
        let sample_json = r#"{"data": [{"id": "p.V7VY2", "attributes": {"name": "Synthwave Mix", "description": {"standard": "Mis favoritos"}}}]}"#;
        let result = parse_user_playlists_json(sample_json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "p.V7VY2");
        assert_eq!(result[0].name, "Synthwave Mix");
        assert_eq!(result[0].description, "Mis favoritos");
    }
}