use reqwest::Client;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
}

pub fn parse_lrc(raw: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for (idx, raw_line) in raw.lines().enumerate() {
        let s = raw_line.trim();
        if s.is_empty() { continue; }
        if let Some(close) = s.find(']') {
            if s.starts_with('[') {
                let tag = &s[1..close];
                let text = s[close + 1..].trim();
                if let Some((m_str, s_str)) = tag.split_once(':') {
                    if let (Ok(mins), Ok(secs)) = (m_str.parse::<u64>(), s_str.parse::<f64>()) {
                        let time_ms = mins * 60_000 + (secs * 1000.0) as u64;
                        if !text.is_empty() { lines.push(LyricLine { time_ms, text: text.to_string() }); }
                        continue;
                    }
                }
            }
        }
        lines.push(LyricLine { time_ms: idx as u64 * 4000, text: s.to_string() });
    }
    lines
}

pub async fn fetch_track_lyrics(http: &Client, title: &str, artist: &str) -> Vec<LyricLine> {
    let clean_title = title.split(['(', '[']).next().unwrap_or(title).trim();
    let q = format!("{artist} {clean_title}");
    if let Ok(resp) = http.get("https://lrclib.net/api/search")
        .query(&[("q", &q)])
        .timeout(std::time::Duration::from_secs(4))
        .send().await
    {
        if let Ok(val) = resp.json::<Value>().await {
            if let Some(arr) = val.as_array() {
                // First pass: check syncedLyrics
                for item in arr {
                    if let Some(synced) = item.get("syncedLyrics").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                        let parsed = parse_lrc(synced);
                        if !parsed.is_empty() { return parsed; }
                    }
                }

                // Second pass: check plainLyrics
                for item in arr {
                    if let Some(plain) = item.get("plainLyrics").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                        let parsed = parse_lrc(plain);
                        if !parsed.is_empty() { return parsed; }
                    }
                }
            }
        }
    }
    vec![LyricLine { time_ms: 0, text: "♪ Letra no disponible para esta pista.".to_string() }]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_lrc() {
        let out = parse_lrc("[00:12.50] Primera línea\n[01:02.00] Segunda línea");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].time_ms, 12_500);
        assert_eq!(out[1].text, "Segunda línea");
    }
}