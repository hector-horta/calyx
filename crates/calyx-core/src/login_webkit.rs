use std::path::PathBuf;
use std::time::{Duration, Instant};
use crate::error::{CalyxError, CalyxResult};

pub fn parse_binary_cookies(data: &[u8]) -> Option<String> {
    if data.len() < 8 || &data[0..4] != b"cook" {
        return None;
    }

    let num_pages = u32::from_be_bytes(data[4..8].try_into().ok()?) as usize;
    let mut page_sizes = Vec::with_capacity(num_pages);
    let mut pos = 8;

    for _ in 0..num_pages {
        if pos + 4 > data.len() { return None; }
        page_sizes.push(u32::from_be_bytes(data[pos..pos+4].try_into().ok()?) as usize);
        pos += 4;
    }

    for page_size in page_sizes {
        if pos + page_size > data.len() { break; }
        let page = &data[pos..pos + page_size];
        pos += page_size;

        if page.len() < 8 || page[0..4] != [0x00, 0x00, 0x01, 0x00] {
            continue;
        }

        let num_cookies = u32::from_le_bytes(page[4..8].try_into().ok()?) as usize;
        let mut cookie_offsets = Vec::with_capacity(num_cookies);
        let mut cpos = 8;

        for _ in 0..num_cookies {
            if cpos + 4 > page.len() { break; }
            cookie_offsets.push(u32::from_le_bytes(page[cpos..cpos+4].try_into().ok()?) as usize);
            cpos += 4;
        }

        for offset in cookie_offsets {
            if offset + 32 > page.len() { continue; }
            let cookie = &page[offset..];
            let size = u32::from_le_bytes(cookie[0..4].try_into().unwrap_or_default()) as usize;
            if size > cookie.len() || size < 32 { continue; }

            let url_off = u32::from_le_bytes(cookie[16..20].try_into().unwrap_or_default()) as usize;
            let name_off = u32::from_le_bytes(cookie[20..24].try_into().unwrap_or_default()) as usize;
            let val_off = u32::from_le_bytes(cookie[28..32].try_into().unwrap_or_default()) as usize;

            let read_str = |start: usize| -> Option<&str> {
                if start >= size { return None; }
                let slice = &cookie[start..size];
                let null_pos = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                std::str::from_utf8(&slice[..null_pos]).ok()
            };

            let name = read_str(name_off).unwrap_or("");
            let domain = read_str(url_off).unwrap_or("");
            let value = read_str(val_off).unwrap_or("");

            if name == "media-user-token" && (domain.contains("apple.com") || domain.contains("mzstatic.com")) && !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub fn find_webkit_cookie_files() -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut files = Vec::new();
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        let h = PathBuf::from(home);
        for sub in ["Library/Cookies/Cookies.binarycookies", "Library/Application Support/Orion/Defaults/Cookies/Cookies.binarycookies"] {
            let p = h.join(sub);
            if p.is_file() { files.push(p); }
        }
    }

    #[cfg(target_os = "linux")]
    if let Ok(home) = std::env::var("HOME") {
        let ep = PathBuf::from(home).join(".local/share/epiphany/cookies.sqlite");
        if ep.is_file() { files.push(ep); }
    }
    files
}

pub fn extract_token_from_webkit() -> Option<String> {
    for path in find_webkit_cookie_files() {
        if path.extension().and_then(|s| s.to_str()) == Some("binarycookies") {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Some(token) = parse_binary_cookies(&bytes) {
                    return Some(token);
                }
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("sqlite") {
            if let Some(token) = crate::login_firefox::extract_token_from_db_file(&path) {
                return Some(token);
            }
        }
    }
    None
}

pub async fn wait_for_webkit_login(timeout: Duration) -> CalyxResult<String> {
    crate::login_firefox::open_url_in_system_browser("https://music.apple.com")?;

    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(token) = extract_token_from_webkit() {
            return Ok(token);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(CalyxError::Auth("Timeout en Safari/WebKit. Si macOS bloqueó permisos, usa '?' para modo manual.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_binary_cookies_synthetic() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"cook");
        buf.extend_from_slice(&1u32.to_be_bytes()); // 1 page
        let mut page = vec![0x00, 0x00, 0x01, 0x00];
        page.extend_from_slice(&1u32.to_le_bytes()); // 1 cookie
        page.extend_from_slice(&12u32.to_le_bytes()); // offset = 12

        let mut cookie = vec![0u8; 32];
        let mut data = Vec::new();
        let u_off = 32 + data.len(); data.extend_from_slice(b".music.apple.com\0");
        let n_off = 32 + data.len(); data.extend_from_slice(b"media-user-token\0");
        let v_off = 32 + data.len(); data.extend_from_slice(b"safari_test_123\0");
        let total_size = 32 + data.len();

        cookie[0..4].copy_from_slice(&(total_size as u32).to_le_bytes());
        cookie[16..20].copy_from_slice(&(u_off as u32).to_le_bytes());
        cookie[20..24].copy_from_slice(&(n_off as u32).to_le_bytes());
        cookie[28..32].copy_from_slice(&(v_off as u32).to_le_bytes());
        cookie.extend_from_slice(&data);

        page.extend_from_slice(&cookie);
        buf.extend_from_slice(&(page.len() as u32).to_be_bytes());
        buf.extend_from_slice(&page);

        assert_eq!(parse_binary_cookies(&buf), Some("safari_test_123".to_string()));
    }
}
