use crate::error::{CalyxError, CalyxResult};
use crate::login::LoginBrowserGuard;
use crate::models::Track;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

#[derive(Clone)]
pub struct AppleMusicClient {
    pub(crate) http: Client,
    pub(crate) storefront: String,
    pub(crate) developer_token: Option<String>,
    pub(crate) media_user_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ItunesSearchPayload {
    #[serde(default)]
    results: Vec<ItunesSongItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItunesSongItem {
    track_id: Option<u64>,
    collection_id: Option<u64>,
    track_name: Option<String>,
    artist_name: Option<String>,
    collection_name: Option<String>,
    track_time_millis: Option<u64>,
    preview_url: Option<String>,
    artwork_url100: Option<String>,
}

impl AppleMusicClient {
    pub fn new(storefront: impl Into<String>) -> CalyxResult<Self> {
        let sf = storefront.into().trim().to_lowercase();
        if sf.len() != 2 || !sf.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(CalyxError::Validation(
                "Storefront must be a 2-letter ISO country code (e.g. 'cl', 'us')".to_string(),
            ));
        }

        let http = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .connect_timeout(std::time::Duration::from_secs(6))
            .timeout(std::time::Duration::from_secs(18))
            .build()?;

        Ok(Self {
            http,
            storefront: sf,
            developer_token: None,
            media_user_token: None,
        })
    }

    pub fn set_tokens(&mut self, dev_token: Option<String>, user_token: Option<String>) {
        self.developer_token = dev_token;
        self.media_user_token = user_token;
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub async fn search_catalog(&self, term: &str, limit: usize) -> CalyxResult<Vec<Track>> {
        let clean_term = term.trim();
        if clean_term.is_empty() {
            return Err(CalyxError::Validation(
                "Search query cannot be empty".to_string(),
            ));
        }

        info!(
            "Searching Apple Music catalog for '{}' (storefront={})",
            clean_term, self.storefront
        );

        if let Some(dev_tok) = self.developer_token.as_deref().filter(|s| !s.is_empty()) {
            let url = format!(
                "https://amp-api.music.apple.com/v1/catalog/{}/search",
                self.storefront
            );
            let lim_s = limit.clamp(1, 25).to_string();
            if let Ok(resp) = self
                .http
                .get(&url)
                .query(&[
                    ("term", clean_term),
                    ("types", "songs"),
                    ("limit", lim_s.as_str()),
                ])
                .header("Authorization", format!("Bearer {dev_tok}"))
                .header("Origin", "https://music.apple.com")
                .header("Referer", "https://music.apple.com/")
                .send()
                .await
                && let Ok(val) = resp.json::<serde_json::Value>().await
                && let Some(songs_obj) = val.pointer("/results/songs")
                && let Ok(list) = crate::library::parse_library_songs_json(
                    &songs_obj.to_string(),
                    &self.storefront,
                )
                && !list.is_empty()
            {
                return Ok(list);
            }
        }

        // Fallback to itunes.apple.com/search
        // reqwest percent-encodes each value, so accents and CJK titles survive.
        let lim = limit.clamp(1, 50).to_string();
        let resp = self
            .http
            .get("https://itunes.apple.com/search")
            .query(&[
                ("media", "music"),
                ("entity", "song"),
                ("country", self.storefront.as_str()),
                ("term", clean_term),
                ("limit", lim.as_str()),
            ])
            .send()
            .await?;
        let payload: ItunesSearchPayload = resp.json().await?;

        let mut tracks = Vec::new();
        for item in payload.results {
            if let (Some(id), Some(title)) = (item.track_id, item.track_name) {
                let track = Track {
                    id: id.to_string(),
                    title,
                    artist: item
                        .artist_name
                        .unwrap_or_else(|| "Unknown Artist".to_string()),
                    album: item
                        .collection_name
                        .unwrap_or_else(|| "Unknown Album".to_string()),
                    duration_ms: item.track_time_millis.unwrap_or(0),
                    storefront: self.storefront.clone(),
                    preview_url: item.preview_url,
                    artwork_url: item.artwork_url100,
                    hls_manifest_url: None,
                    is_lossless: true,
                    album_id: item.collection_id.map(|id| id.to_string()),
                };
                if track.validate().is_ok() {
                    tracks.push(track);
                }
            }
        }

        Ok(tracks)
    }

    pub async fn sync_developer_token(&mut self) -> CalyxResult<String> {
        let token = crate::auth::fetch_apple_developer_token(&self.http).await?;
        self.developer_token = Some(token.clone());
        Ok(token)
    }

    pub async fn login_and_verify_user(
        &mut self,
    ) -> CalyxResult<(String, String, String, Option<LoginBrowserGuard>)> {
        let dev_token = match &self.developer_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => self.sync_developer_token().await?,
        };
        if let Some(existing) = self.media_user_token.clone().filter(|t| !t.is_empty())
            && crate::login::token_is_live(&self.http, &dev_token, &existing).await
        {
            return Ok((dev_token, existing, self.storefront.clone(), None));
        }
        let (extracted_dev, user_token, guard) =
            crate::login::interactive_apple_login(&self.http, &self.storefront, &dev_token).await?;
        let dev_token = extracted_dev.unwrap_or(dev_token);
        self.developer_token = Some(dev_token.clone());
        let verified_sf = self
            .verify_user_storefront(&dev_token, &user_token)
            .await
            .unwrap_or_else(|| self.storefront.clone());
        self.media_user_token = Some(user_token.clone());
        self.storefront = verified_sf.clone();
        Ok((dev_token, user_token, verified_sf, Some(guard)))
    }

    async fn verify_user_storefront(&self, dev_token: &str, user_token: &str) -> Option<String> {
        let resp = self
            .http
            .get("https://amp-api.music.apple.com/v1/me/storefront")
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .send()
            .await
            .ok()?;
        let val: serde_json::Value = resp.json().await.ok()?;
        val.pointer("/data/0/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase())
    }

    pub async fn fetch_stream_bytes(&self, url: &str) -> CalyxResult<Vec<u8>> {
        if !url.starts_with("https://") {
            return Err(CalyxError::Validation(
                "Stream URL must use HTTPS".to_string(),
            ));
        }
        let bytes = self.http.get(url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storefront_validation() {
        assert!(AppleMusicClient::new("cl").is_ok() && AppleMusicClient::new("US").is_ok());
        assert!(AppleMusicClient::new("12").is_err() && AppleMusicClient::new("chile").is_err());
    }

    #[tokio::test]
    async fn test_rejects_empty_search_term() {
        assert!(
            AppleMusicClient::new("cl")
                .expect("client")
                .search_catalog("   ", 10)
                .await
                .is_err()
        );
    }
}
