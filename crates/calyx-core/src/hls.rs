use crate::drm::PsshBox;
use crate::error::{CalyxError, CalyxResult};

#[derive(Debug, Clone)]
pub struct HlsStreamPlan {
    pub pssh: Option<PsshBox>,
    pub key_uri: Option<String>,
    pub init_segment_url: Option<String>,
    pub media_segment_urls: Vec<String>,
}

impl HlsStreamPlan {
    pub fn parse_m3u8(manifest_text: &str, base_url: &str) -> CalyxResult<Self> {
        if !manifest_text.contains("#EXTM3U") {
            return Err(CalyxError::Validation(
                "Invalid HLS manifest: missing #EXTM3U header".to_string(),
            ));
        }

        let mut pssh: Option<PsshBox> = None;
        let mut key_uri: Option<String> = None;
        let mut init_segment_url: Option<String> = None;
        let mut media_segment_urls = Vec::new();

        for raw_line in manifest_text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("#EXT-X-KEY:") || line.starts_with("#EXT-X-SESSION-KEY:") {
                if let Some(uri) = extract_quoted_attr(line, "URI=") {
                    if let Some(b64_pssh) = uri
                        .strip_prefix("data:text/plain;base64,")
                        .or_else(|| uri.strip_prefix("data:;base64,"))
                    {
                        pssh = PsshBox::from_base64(b64_pssh).ok();
                    }
                    key_uri = Some(uri);
                }
            } else if line.starts_with("#EXT-X-MAP:") {
                if let Some(map_uri) = extract_quoted_attr(line, "URI=") {
                    init_segment_url = Some(resolve_relative_url(base_url, &map_uri));
                }
            } else if !line.starts_with('#') {
                media_segment_urls.push(resolve_relative_url(base_url, line));
            }
        }

        Ok(Self {
            pssh,
            key_uri,
            init_segment_url,
            media_segment_urls,
        })
    }

    pub fn extract_variant_url(manifest_text: &str, base_url: &str) -> Option<String> {
        let mut expect_uri = false;
        for raw_line in manifest_text.lines() {
            let line = raw_line.trim();
            if line.starts_with("#EXT-X-STREAM-INF:") {
                expect_uri = true;
            } else if expect_uri && !line.is_empty() && !line.starts_with('#') {
                return Some(resolve_relative_url(base_url, line));
            }
        }
        None
    }
}

fn extract_quoted_attr(line: &str, attr_key: &str) -> Option<String> {
    let idx = line.find(attr_key)?;
    let after = &line[idx + attr_key.len()..];
    let stripped = after.strip_prefix('"')?;
    let end = stripped.find('"')?;
    Some(stripped[..end].to_string())
}

fn resolve_relative_url(base_url: &str, target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_string();
    }
    let clean_base = match base_url.rfind('/') {
        Some(pos) => &base_url[..=pos],
        None => base_url,
    };
    format!("{}{}", clean_base, target.trim_start_matches('/'))
}

#[cfg(test)]
#[path = "../tests/unit/hls.rs"]
mod tests;
