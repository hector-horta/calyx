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
            return Err(CalyxError::Validation("Invalid HLS manifest: missing #EXTM3U header".to_string()));
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
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    #[test]
    fn test_parse_apple_hls_m3u8() {
        let mut raw_pssh = vec![0u8; 32];
        raw_pssh[4..8].copy_from_slice(b"pssh");
        let b64_pssh = BASE64.encode(&raw_pssh);

        let m3u8 = format!(
            "#EXTM3U\n\
             #EXT-X-VERSION:7\n\
             #EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"data:text/plain;base64,{}\",KEYFORMAT=\"urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed\"\n\
             #EXT-X-MAP:URI=\"init_0.mp4\"\n\
             #EXTINF:4.000,\n\
             seg_0001.m4s\n\
             #EXTINF:4.000,\n\
             seg_0002.m4s\n",
            b64_pssh
        );

        let plan = HlsStreamPlan::parse_m3u8(&m3u8, "https://aod-ssl.itunes.apple.com/streams/master.m3u8")
            .expect("valid m3u8");
        assert!(plan.pssh.is_some());
        assert_eq!(
            plan.init_segment_url.as_deref(),
            Some("https://aod-ssl.itunes.apple.com/streams/init_0.mp4")
        );
        assert_eq!(plan.media_segment_urls.len(), 2);

        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=256000,CODECS=\"mp4a.40.2\"\naudio_256/prog_index.m3u8\n";
        assert_eq!(
            HlsStreamPlan::extract_variant_url(master, "https://aod-ssl.itunes.apple.com/streams/master.m3u8").as_deref(),
            Some("https://aod-ssl.itunes.apple.com/streams/audio_256/prog_index.m3u8")
        );
    }
}