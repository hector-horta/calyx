use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::Deserialize;
use tracing::info;
use crate::drm::WidevineDeviceL3;
use crate::error::{CalyxError, CalyxResult};
use crate::fmp4::decrypt_apple_fmp4_stream;
use crate::hls::HlsStreamPlan;
use crate::login::LoginBrowserGuard;
use crate::models::Track;

#[derive(Clone)]
pub struct AppleMusicClient {
    http: Client,
    storefront: String,
    developer_token: Option<String>,
    media_user_token: Option<String>,
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
        return Err(CalyxError::Validation("Search query cannot be empty".to_string()));
    }

    info!("Searching Apple Music catalog for '{}' (storefront={})", clean_term, self.storefront);

    if let Some(dev_tok) = self.developer_token.as_deref().filter(|s| !s.is_empty()) {
        let url = format!("https://amp-api.music.apple.com/v1/catalog/{}/search", self.storefront);
        let lim_s = limit.clamp(1, 25).to_string();
        if let Ok(resp) = self.http.get(&url)
            .query(&[("term", clean_term), ("types", "songs"), ("limit", lim_s.as_str())])
            .header("Authorization", format!("Bearer {dev_tok}"))
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .send().await
        {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                if let Some(songs_obj) = val.pointer("/results/songs") {
                    if let Ok(list) = crate::library::parse_library_songs_json(&songs_obj.to_string(), &self.storefront) {
                        if !list.is_empty() { return Ok(list); }
                    }
                }
            }
        }
    }

    // Fallback to itunes.apple.com/search
    let encoded_term: String = clean_term.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '+' })
        .collect();

    let url = format!(
        "https://itunes.apple.com/search?media=music&entity=song&country={}&term={}&limit={}",
        self.storefront,
        encoded_term,
        limit.clamp(1, 50)
    );

    let resp = self.http.get(&url).send().await?;
    let payload: ItunesSearchPayload = resp.json().await?;

    let mut tracks = Vec::new();
    for item in payload.results {
        if let (Some(id), Some(title)) = (item.track_id, item.track_name) {
            let track = Track {
                id: id.to_string(),
                title,
                artist: item.artist_name.unwrap_or_else(|| "Unknown Artist".to_string()),
                album: item.collection_name.unwrap_or_else(|| "Unknown Album".to_string()),
                duration_ms: item.track_time_millis.unwrap_or(0),
                storefront: self.storefront.clone(),
                preview_url: item.preview_url,
                artwork_url: item.artwork_url100,
                hls_manifest_url: None,
                is_lossless: true,
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
        if let Some(existing) = self.media_user_token.clone().filter(|t| !t.is_empty()) {
            if crate::login::token_is_live(&self.http, &dev_token, &existing).await {
                return Ok((dev_token, existing, self.storefront.clone(), None));
            }
        }
        let (user_token, guard) =
            crate::login::interactive_apple_login(&self.http, &self.storefront, &dev_token).await?;
        let verified_sf = self
            .verify_user_storefront(&dev_token, &user_token)
            .await
            .unwrap_or_else(|| self.storefront.clone());
        self.media_user_token = Some(user_token.clone());
        self.storefront = verified_sf.clone();
        Ok((dev_token, user_token, verified_sf, Some(guard)))
    }

    pub async fn fetch_full_track_stream(&self, track_id: &str, wvd: &WidevineDeviceL3) -> CalyxResult<Vec<u8>> {
        let dev_token = self.developer_token.as_deref().filter(|s| !s.is_empty())
            .ok_or_else(|| CalyxError::Auth("Falta Developer Token".to_string()))?;
        let user_token = self.media_user_token.as_deref().filter(|s| !s.is_empty())
            .ok_or_else(|| CalyxError::Auth("Falta Media-User-Token".to_string()))?;

        let (hls_url, license_url) = self.resolve_webplay_manifest(track_id, dev_token, user_token).await?;
        let first_m3u8 = self.http.get(&hls_url).send().await?.text().await?;
        let (media_m3u8, media_base_url) = match HlsStreamPlan::extract_variant_url(&first_m3u8, &hls_url) {
            Some(v_url) => (self.http.get(&v_url).send().await?.text().await?, v_url),
            None => (first_m3u8, hls_url),
        };

        let plan = HlsStreamPlan::parse_m3u8(&media_m3u8, &media_base_url)?;
        let pssh = plan.pssh.as_ref().ok_or_else(|| CalyxError::Drm("Manifiesto HLS sin PSSH".to_string()))?;
        let key_uri = plan.key_uri.as_deref().unwrap_or("");
        let is_cenc = media_m3u8.contains("SAMPLE-AES-CTR") || media_base_url.contains("ctrp");
        let (challenge, lic_req) = match (&wvd.rsa_d, &wvd.rsa_n) {
            (Some(d), Some(n)) => wvd.build_signed_challenge(pssh, d, n),
            _ => (wvd.build_license_challenge(pssh)?, Vec::new()),
        };
        let lic_b64 = self.request_widevine_license(&license_url, track_id, key_uri, pssh, &challenge, dev_token, user_token).await?;
        let track_key = wvd.extract_content_key_from_license(&lic_b64, &lic_req)?;

        let mut full_audio = Vec::new();
        let mut seen_urls = Vec::new();
        for seg_url in plan.init_segment_url.iter().chain(plan.media_segment_urls.iter().take(64)) {
            if !seen_urls.contains(seg_url) {
                seen_urls.push(seg_url.clone());
                full_audio.extend_from_slice(&self.fetch_stream_bytes(seg_url).await?);
            }
        }

        if full_audio.is_empty() {
            return Err(CalyxError::Audio("Stream HLS vacío".to_string()));
        }

        decrypt_apple_fmp4_stream(&mut full_audio, &track_key, is_cenc)?;
        Ok(full_audio)
    }

async fn resolve_webplay_manifest(&self, track_id: &str, dev_token: &str, user_token: &str) -> CalyxResult<(String, String)> {
    let payload = if track_id.starts_with("i.") {
        serde_json::json!({ "universalLibraryId": track_id })
    } else {
        serde_json::json!({ "salableAdamId": track_id })
    };

    let resp = self.http.post("https://play.itunes.apple.com/WebObjects/MZPlay.woa/wa/webPlayback")
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Accept", "application/json, text/plain, */*")
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .json(&payload).send().await?;

    let val = self.parse_apple_json(resp, "webPlayback").await?;
    let song = val.pointer("/songList/0").ok_or_else(|| CalyxError::Drm("webPlayback sin activos".to_string()))?;
    let ctrp_url = song.get("assets").and_then(|v| v.as_array()).and_then(|arr| {
        arr.iter().find(|a| a.get("flavor").and_then(|f| f.as_str()) == Some("28:ctrp256"))
            .or_else(|| arr.iter().find(|a| a.get("flavor").and_then(|f| f.as_str()) == Some("32:ctrp64")))
            .or_else(|| arr.first())
            .and_then(|a| a.get("URL").and_then(|u| u.as_str()))
    });
    let hls_url = ctrp_url.or_else(|| song.get("hls-playlist-url").and_then(|v| v.as_str()))
        .ok_or_else(|| CalyxError::Drm("Sin URL HLS en webPlay".to_string()))?.to_string();
    let license_url = song.get("hls-key-server-url").and_then(|v| v.as_str())
        .unwrap_or("https://play.itunes.apple.com/WebObjects/MZPlay.woa/wa/acquireWebPlaybackLicense").to_string();

    Ok((hls_url, license_url))
}

    async fn request_widevine_license(&self, license_url: &str, track_id: &str, key_uri: &str, pssh: &crate::drm::PsshBox, challenge: &[u8], dev_token: &str, user_token: &str) -> CalyxResult<String> {
        let uri_field = if !key_uri.is_empty() { key_uri.to_string() } else { format!("data:text/plain;base64,{}", BASE64.encode(&pssh.raw_bytes)) };
        let body = serde_json::json!({
            "adamId": track_id, "challenge": BASE64.encode(challenge), "isLibrary": track_id.starts_with("i."),
            "key-system": "com.widevine.alpha", "uri": uri_field, "user-initiated": true
        });
        let resp = self.http.post(license_url)
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .json(&body).send().await?;
        let val = self.parse_apple_json(resp, "acquireLicense").await?;
        val.get("license").and_then(|v| v.as_str()).map(|s| s.to_string())
            .ok_or_else(|| CalyxError::Drm("Licencia Widevine rechazada o vacía".to_string()))
    }

    async fn parse_apple_json(&self, resp: reqwest::Response, endpoint: &str) -> CalyxResult<serde_json::Value> {
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(CalyxError::Auth(format!("Sesión Apple expirada (HTTP {status}). Presiona 'u' para login.")));
            }
            return Err(CalyxError::Drm(format!("Apple {endpoint} HTTP {status}: {text}")));
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        Ok(val)
    }

    async fn verify_user_storefront(&self, dev_token: &str, user_token: &str) -> Option<String> {
        let resp = self.http.get("https://amp-api.music.apple.com/v1/me/storefront")
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .send().await.ok()?;
        let val: serde_json::Value = resp.json().await.ok()?;
        val.pointer("/data/0/id").and_then(|v| v.as_str()).map(|s| s.to_lowercase())
    }

    pub async fn fetch_stream_bytes(&self, url: &str) -> CalyxResult<Vec<u8>> {
        if !url.starts_with("https://") {
            return Err(CalyxError::Validation("Stream URL must use HTTPS".to_string()));
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
        assert!(AppleMusicClient::new("cl").is_ok());
        assert!(AppleMusicClient::new("US").is_ok());
        assert!(AppleMusicClient::new("12").is_err());
        assert!(AppleMusicClient::new("chile").is_err());
    }

    #[tokio::test]
    async fn test_rejects_empty_search_term() {
        let client = AppleMusicClient::new("cl").expect("client");
        assert!(client.search_catalog("   ", 10).await.is_err());
    }
}
