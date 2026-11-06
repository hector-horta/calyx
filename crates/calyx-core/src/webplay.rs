//! Full-track playback: webPlayback manifest, Widevine license and the
//! decrypted HLS download. Split out of `api.rs`; behaviour unchanged.

use crate::api::AppleMusicClient;
use crate::drm::WidevineDeviceL3;
use crate::error::{CalyxError, CalyxResult};
use crate::fmp4::decrypt_apple_fmp4_stream;
use crate::hls::HlsStreamPlan;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

impl AppleMusicClient {
    pub async fn fetch_full_track_stream(
        &self,
        track_id: &str,
        wvd: &WidevineDeviceL3,
    ) -> CalyxResult<Vec<u8>> {
        let dev_token = self
            .developer_token
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CalyxError::Auth("Falta Developer Token".to_string()))?;
        let user_token = self
            .media_user_token
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CalyxError::Auth("Falta Media-User-Token".to_string()))?;

        let (hls_url, license_url) = self
            .resolve_webplay_manifest(track_id, dev_token, user_token)
            .await?;
        let first_m3u8 = self.http.get(&hls_url).send().await?.text().await?;
        let (media_m3u8, media_base_url) =
            match HlsStreamPlan::extract_variant_url(&first_m3u8, &hls_url) {
                Some(v_url) => (self.http.get(&v_url).send().await?.text().await?, v_url),
                None => (first_m3u8, hls_url),
            };

        let plan = HlsStreamPlan::parse_m3u8(&media_m3u8, &media_base_url)?;
        let pssh = plan
            .pssh
            .as_ref()
            .ok_or_else(|| CalyxError::Drm("Manifiesto HLS sin PSSH".to_string()))?;
        let key_uri = plan.key_uri.as_deref().unwrap_or("");
        let is_cenc = media_m3u8.contains("SAMPLE-AES-CTR") || media_base_url.contains("ctrp");
        let (challenge, lic_req) = match (&wvd.rsa_d, &wvd.rsa_n) {
            (Some(d), Some(n)) => wvd.build_signed_challenge(pssh, d, n),
            _ => (wvd.build_license_challenge(pssh)?, Vec::new()),
        };
        let lic_b64 = self
            .request_widevine_license(
                &license_url,
                track_id,
                key_uri,
                pssh,
                &challenge,
                (dev_token, user_token),
            )
            .await?;
        let track_key = wvd.extract_content_key_from_license(&lic_b64, &lic_req)?;

        let mut full_audio = Vec::new();
        let mut seen_urls = Vec::new();
        for seg_url in plan
            .init_segment_url
            .iter()
            .chain(plan.media_segment_urls.iter().take(64))
        {
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

    async fn resolve_webplay_manifest(
        &self,
        track_id: &str,
        dev_token: &str,
        user_token: &str,
    ) -> CalyxResult<(String, String)> {
        let payload = if track_id.starts_with("i.") {
            serde_json::json!({ "universalLibraryId": track_id })
        } else {
            serde_json::json!({ "salableAdamId": track_id })
        };

        let resp = self
            .http
            .post("https://play.itunes.apple.com/WebObjects/MZPlay.woa/wa/webPlayback")
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .json(&payload)
            .send()
            .await?;

        let val = self.parse_apple_json(resp, "webPlayback").await?;
        let song = val.pointer("/songList/0").ok_or_else(|| {
            let detail = val
                .get("customerMessage")
                .or_else(|| val.get("failureType"))
                .and_then(|v| v.as_str())
                .unwrap_or("sin activos");
            CalyxError::Drm(format!("webPlayback: {detail}"))
        })?;
        let ctrp_url = song
            .get("assets")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|a| a.get("flavor").and_then(|f| f.as_str()) == Some("28:ctrp256"))
                    .or_else(|| {
                        arr.iter()
                            .find(|a| a.get("flavor").and_then(|f| f.as_str()) == Some("32:ctrp64"))
                    })
                    .or_else(|| arr.first())
                    .and_then(|a| a.get("URL").and_then(|u| u.as_str()))
            });
        let hls_url = ctrp_url
            .or_else(|| song.get("hls-playlist-url").and_then(|v| v.as_str()))
            .ok_or_else(|| CalyxError::Drm("Sin URL HLS en webPlay".to_string()))?
            .to_string();
        let license_url = song
            .get("hls-key-server-url")
            .and_then(|v| v.as_str())
            .unwrap_or(
                "https://play.itunes.apple.com/WebObjects/MZPlay.woa/wa/acquireWebPlaybackLicense",
            )
            .to_string();

        Ok((hls_url, license_url))
    }

    /// `tokens` = (developer token, media-user token).
    async fn request_widevine_license(
        &self,
        license_url: &str,
        track_id: &str,
        key_uri: &str,
        pssh: &crate::drm::PsshBox,
        challenge: &[u8],
        tokens: (&str, &str),
    ) -> CalyxResult<String> {
        let (dev_token, user_token) = tokens;
        let uri_field = if !key_uri.is_empty() {
            key_uri.to_string()
        } else {
            format!("data:text/plain;base64,{}", BASE64.encode(&pssh.raw_bytes))
        };
        let body = serde_json::json!({
            "adamId": track_id, "challenge": BASE64.encode(challenge), "isLibrary": track_id.starts_with("i."),
            "key-system": "com.widevine.alpha", "uri": uri_field, "user-initiated": true
        });
        let resp = self
            .http
            .post(license_url)
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .json(&body)
            .send()
            .await?;
        let val = self.parse_apple_json(resp, "acquireLicense").await?;
        val.get("license")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CalyxError::Drm("Licencia Widevine rechazada o vacía".to_string()))
    }

    pub(crate) async fn parse_apple_json(
        &self,
        resp: reqwest::Response,
        endpoint: &str,
    ) -> CalyxResult<serde_json::Value> {
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(CalyxError::Auth(format!(
                    "Sesión Apple expirada (HTTP {status}). Presiona 'u' para login."
                )));
            }
            return Err(CalyxError::Drm(format!(
                "Apple {endpoint} HTTP {status}: {text}"
            )));
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        Ok(val)
    }
}
