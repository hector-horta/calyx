// lyrics_ttml.rs

use crate::lyrics::LyricLine;
use reqwest::Client;
use serde_json::Value;

pub fn parse_ttml_time(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();

        match parts.len() {
            2 => {
                let m: f64 = parts[0].parse().ok()?;
                let sec: f64 = parts[1].parse().ok()?;
                Some(((m * 60.0 + sec) * 1000.0) as u64)
            }
            3 => {
                let h: f64 = parts[0].parse().ok()?;
                let m: f64 = parts[1].parse().ok()?;
                let sec: f64 = parts[2].parse().ok()?;
                Some(((h * 3600.0 + m * 60.0 + sec) * 1000.0) as u64)
            }
            _ => None,
        }
    } else {
        let sec: f64 = s.parse().ok()?;
        Some((sec * 1000.0) as u64)
    }
}

pub fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

pub fn strip_xml_tags(s: &str) -> String {
    let mut out = String::new();
    let mut inside_tag = false;

    for c in s.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            out.push(c);
        }
    }

    out
}

pub fn parse_ttml(raw: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    let mut current_text = raw;

    while let Some(p_start) = current_text.find("<p ") {
        let rest = &current_text[p_start..];
        if let Some(tag_end_pos) = rest.find('>') {
            let tag_attrs = &rest[..tag_end_pos];

            if let Some(p_close_pos) = rest[tag_end_pos..].find("</p>") {
                let inner_content = &rest[tag_end_pos + 1..tag_end_pos + p_close_pos];
                let text = decode_xml_entities(&strip_xml_tags(inner_content))
                    .trim()
                    .to_string();

                if let Some(time_start) = tag_attrs.find("begin=\"") {
                    let time_str = &tag_attrs[time_start + 7..];

                    if let Some(quote_end_pos) = time_str.find('"') {
                        let time_str = &time_str[..quote_end_pos];
                        if let Some(time_ms) = parse_ttml_time(time_str)
                            && !text.is_empty()
                        {
                            lines.push(LyricLine::new(time_ms, text));
                        }
                    }
                }

                current_text = &rest[tag_end_pos + p_close_pos + 4..];
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if !lines.is_empty() && lines.first().map(|l| l.time_ms >= 3000).unwrap_or(false) {
        lines.insert(0, LyricLine::new(0, "•••"));
    }

    lines
}

pub async fn fetch_apple_music_lyrics(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    storefront: &str,
    song_id: &str,
) -> Option<Vec<LyricLine>> {
    if dev_token.is_empty() || song_id.is_empty() {
        return None;
    }

    let url =
        format!("https://amp-api.music.apple.com/v1/catalog/{storefront}/songs/{song_id}/lyrics");

    let resp = http
        .get(&url)
        .header("Authorization", format!("Bearer {}", dev_token))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .timeout(std::time::Duration::from_secs(6))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let val: Value = resp.json().await.ok()?;
    let ttml = val
        .get("data")?
        .get(0)?
        .get("attributes")?
        .get("ttml")?
        .as_str()?;

    let parsed = parse_ttml(ttml);
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ttml_time() {
        assert_eq!(parse_ttml_time("1.319"), Some(1319));
        assert_eq!(parse_ttml_time("1:02.635"), Some(62635));
        assert_eq!(parse_ttml_time("01:02.635"), Some(62635));
        assert_eq!(parse_ttml_time("1:02:03.500"), Some(3723500));
    }

    #[test]
    fn test_parse_ttml_japanese_sample() {
        let sample = r#"<tt xml:lang="ja"><body><div>
            <p begin="1.319" end="2.555">吠えるわ</p>
            <p begin="1:02.635" end="1:10.372">心静かに騒ぐ &amp; <span>抱きしめたい</span></p>
        </div></body></tt>"#;
        let lines = parse_ttml(sample);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].time_ms, 1319);
        assert_eq!(lines[0].text, "吠えるわ");
        assert_eq!(lines[1].time_ms, 62635);
        assert_eq!(lines[1].text, "心静かに騒ぐ & 抱きしめたい");
    }
}
