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
                        let val = if text.is_empty() { "•••" } else { text };
                        if val != "•••" || lines.last().map(|l: &LyricLine| l.text != "•••").unwrap_or(true) {
                            lines.push(LyricLine { time_ms, text: val.to_string() });
                        }
                        continue;
                    }
                }
            }
        }
        lines.push(LyricLine { time_ms: idx as u64 * 4000, text: s.to_string() });
    }
    if lines.first().map(|l| l.time_ms >= 3000).unwrap_or(false) {
        lines.insert(0, LyricLine { time_ms: 0, text: "•••".to_string() });
    }
    lines
}

pub async fn fetch_track_lyrics(http: &Client, title: &str, artist: &str) -> Vec<LyricLine> {
    let clean_title = title.split(['(', '[']).next().unwrap_or(title).split(" - ").next().unwrap_or(title).trim();
    let q = format!("{artist} {clean_title}");
    if let Ok(resp) = http.get("https://lrclib.net/api/search")
        .query(&[("q", &q)])
        .timeout(std::time::Duration::from_secs(7))
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

impl LyricLine {
    pub fn karaoke_parts(&self, cur_ms: u64, next_ms: u64, max_chars: usize) -> (String, String, String) {
        let gap = next_ms.saturating_sub(self.time_ms).max(1200);
        let elapsed = cur_ms.saturating_sub(self.time_ms);
        if self.text == "•••" {
            return match ((elapsed * 3) / gap).min(2) {
                0 => ("● ".to_string(), "○ ".to_string(), "○".to_string()),
                1 => ("● ● ".to_string(), "○".to_string(), String::new()),
                _ => ("● ● ●".to_string(), String::new(), String::new()),
            };
        }
        let mut chars: Vec<char> = self.text.chars().collect();
        if chars.len() > max_chars {
            chars.truncate(max_chars.saturating_sub(1));
            chars.push('…');
        }
        let clean: String = chars.into_iter().collect();
        let total_chars = clean.chars().count();
        if total_chars == 0 {
            return (String::new(), String::new(), String::new());
        }

        let gap = next_ms.saturating_sub(self.time_ms).max(1200);
        let sing_dur = ((gap as f64) * 0.85).clamp(800.0, 5000.0);
        let elapsed = cur_ms.saturating_sub(self.time_ms);
        let ratio = (elapsed as f64 / sing_dur).clamp(0.0, 1.0);
        let target_char = ((total_chars as f64) * ratio).floor() as usize;

        let mut sung = String::new();
        let mut active = String::new();
        let mut unsung = String::new();
        let mut seen = 0usize;

        for word in clean.split_inclusive(' ') {
            let w_len = word.chars().count();
            if ratio >= 1.0 || seen + w_len <= target_char {
                sung.push_str(word);
            } else if active.is_empty() && seen <= target_char {
                active.push_str(word);
            } else {
                unsung.push_str(word);
            }
            seen += w_len;
        }

        if ratio >= 1.0 && gap >= 4500 && cur_ms < next_ms {
            let wait_total = next_ms.saturating_sub(self.time_ms + sing_dur as u64).max(1);
            let wait_elap = cur_ms.saturating_sub(self.time_ms + sing_dur as u64);
            let dots = match (wait_elap * 3) / wait_total {
                0 => "  ● ○ ○",
                1 => "  ● ● ○",
                _ => "  ● ● ●",
            };
            unsung.push_str(dots);
        }
        (sung, active, unsung)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_lrc() {
        let out = parse_lrc("[00:12.50] Primera línea\n[01:02.00] Segunda línea");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].text, "•••");
        assert_eq!(out[1].time_ms, 12_500);
        let (s, a, u) = out[1].karaoke_parts(13_500, 16_500, 40);
        assert!(!s.is_empty() || !a.is_empty() || !u.is_empty());
    }
}