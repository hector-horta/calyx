use std::fs;
use std::path::{Path, PathBuf};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::drm::WidevineDeviceL3;
use crate::error::{CalyxError, CalyxResult};

const AUTH_FILE_PATH: &str = "calyx_auth.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionConfig {
    pub storefront: String,
    pub developer_token: Option<String>,
    pub media_user_token: Option<String>,
    pub wvd_path: Option<String>,
}

impl SessionConfig {
    pub fn load_or_default(default_storefront: &str) -> Self {
        let mut cfg = if Path::new(AUTH_FILE_PATH).exists() {
            fs::read_to_string(AUTH_FILE_PATH)
                .ok()
                .and_then(|raw| serde_json::from_str::<SessionConfig>(&raw).ok())
                .unwrap_or_default()
        } else {
            SessionConfig::default()
        };

        if cfg.storefront.trim().is_empty() {
            cfg.storefront = std::env::var("CALYX_STOREFRONT")
                .unwrap_or_else(|_| default_storefront.to_string());
        }
        if cfg.developer_token.is_none() {
            cfg.developer_token = std::env::var("CALYX_DEVELOPER_TOKEN").ok();
        }
        if cfg.media_user_token.is_none() {
            cfg.media_user_token = std::env::var("CALYX_MEDIA_USER_TOKEN").ok();
        }
        if cfg.wvd_path.is_none() {
            cfg.wvd_path = std::env::var("CALYX_WVD_PATH")
                .ok()
                .or_else(discover_local_wvd_path);
        }
        cfg
    }

    pub fn save(&self) -> CalyxResult<()> {
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(AUTH_FILE_PATH, serialized)
            .map_err(|e| CalyxError::Auth(format!("Failed to write auth config: {}", e)))?;
        Ok(())
    }

    pub fn has_full_stream_credentials(&self) -> bool {
        self.developer_token.as_ref().is_some_and(|t| !t.trim().is_empty())
            && self.media_user_token.as_ref().is_some_and(|t| !t.trim().is_empty())
    }

    pub fn resolve_wvd_path(&self) -> Option<PathBuf> {
        if let Some(p) = self.wvd_path.as_deref().filter(|s| !s.trim().is_empty()) {
            let pb = PathBuf::from(p);
            if pb.exists() {
                return Some(pb);
            }
        }
        discover_local_wvd_path().map(PathBuf::from)
    }

    pub fn load_wvd_device(&self) -> CalyxResult<WidevineDeviceL3> {
        if let Some(path) = self.resolve_wvd_path() {
            if let Ok(raw) = fs::read(&path) {
                if let Ok(dev) = WidevineDeviceL3::from_wvd_bytes(&raw) {
                    return Ok(dev);
                }
            }
        }
        WidevineDeviceL3::builtin()
    }
}

fn discover_local_wvd_path() -> Option<String> {
    for candidate in ["device.wvd", "calyx.wvd", "wvd/device.wvd"] {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("wvd") {
                return path.to_str().map(|s| s.to_string());
            }
        }
    }
    None
}

pub async fn fetch_apple_developer_token(http: &Client) -> CalyxResult<String> {
    info!("Fetching Apple Music web bundle to extract public Developer Token...");
    let html = http
        .get("https://music.apple.com/us/browse")
        .send()
        .await?
        .text()
        .await?;

    let js_path = extract_js_bundle_path(&html)
        .ok_or_else(|| CalyxError::Auth("Could not locate Apple Music JS bundle path in HTML".to_string()))?;

    let js_url = if js_path.starts_with("http") {
        js_path
    } else {
        format!("https://music.apple.com{}", js_path)
    };

    let js_body = http.get(&js_url).send().await?.text().await?;
    extract_jwt_from_bundle(&js_body)
        .ok_or_else(|| CalyxError::Auth("Could not find ES256 Developer JWT in Apple Music bundle".to_string()))
}

pub fn extract_js_bundle_path(html: &str) -> Option<String> {
    for chunk in html.split('"') {
        if (chunk.starts_with("/assets/index") || chunk.starts_with("/assets/main"))
            && chunk.ends_with(".js")
        {
            return Some(chunk.to_string());
        }
    }
    None
}

pub fn extract_jwt_from_bundle(js: &str) -> Option<String> {
    let prefixes = [
        "eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIs",
        "eyJhbGciOiJFUzI1NiIs",
    ];
    for prefix in prefixes {
        if let Some(start) = js.find(prefix) {
            let tail = &js[start..];
            if let Some(end) = tail.find(|c: char| c == '"' || c == '\'' || c.is_whitespace()) {
                let token = &tail[..end];
                if token.split('.').count() == 3 && token.len() > 120 {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_js_and_jwt() {
        let sample_html = r#"<script type="module" crossorigin src="/assets/index-legacy-a1b2c3.js"></script>"#;
        assert_eq!(
            extract_js_bundle_path(sample_html),
            Some("/assets/index-legacy-a1b2c3.js".to_string())
        );

        let fake_jwt = format!(
            "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.{}.{}",
            "a".repeat(80),
            "b".repeat(60)
        );
        let bundle = format!("const devToken = \"{}\";", fake_jwt);
        assert_eq!(extract_jwt_from_bundle(&bundle), Some(fake_jwt));
    }
}