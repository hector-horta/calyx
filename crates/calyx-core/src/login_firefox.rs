use std::path::Path;
use std::time::{Duration, Instant};
use rusqlite::{Connection, OpenFlags};
use crate::browser_discovery::find_gecko_cookie_databases;
use crate::error::{CalyxError, CalyxResult};

pub fn extract_token_from_db_file(db_path: &Path) -> Option<String> {
    let tmp_path = std::env::temp_dir().join(format!("calyx_ff_snap_{}.sqlite", std::process::id()));
    if std::fs::copy(db_path, &tmp_path).is_err() {
        return None;
    }

    let res = (|| {
        let conn = Connection::open_with_flags(&tmp_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
        let mut stmt = conn.prepare(
            "SELECT value FROM moz_cookies 
             WHERE (host LIKE '%apple.com' OR host LIKE '%mzstatic.com') 
               AND name = 'media-user-token' 
             ORDER BY lastAccessed DESC LIMIT 1"
        ).ok()?;
        let mut rows = stmt.query([]).ok()?;
        if let Some(row) = rows.next().ok().flatten() {
            let val: String = row.get(0).ok()?;
            if !val.trim().is_empty() {
                return Some(val);
            }
        }
        None
    })();

    let _ = std::fs::remove_file(&tmp_path);
    res
}

pub fn extract_media_user_token_from_firefox_profiles() -> Option<String> {
    for db in find_gecko_cookie_databases() {
        if let Some(token) = extract_token_from_db_file(&db) {
            return Some(token);
        }
    }
    None
}

pub fn open_url_in_system_browser(url: &str) -> CalyxResult<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on macOS: {e}")))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on Windows: {e}")))?;

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    std::process::Command::new("xdg-open").arg(url).spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on Linux: {e}")))?;

    Ok(())
}

pub async fn wait_for_firefox_login(timeout: Duration) -> CalyxResult<String> {
    open_url_in_system_browser("https://music.apple.com")?;

    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(token) = extract_media_user_token_from_firefox_profiles() {
            return Ok(token);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(CalyxError::Auth("Timeout esperando inicio de sesión en Firefox o fork compatible".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_mock_moz_cookies() {
        let tmp = std::env::temp_dir().join(format!("test_moz_cookies_{}.sqlite", std::process::id()));
        let conn = Connection::open(&tmp).unwrap();
        conn.execute_batch(
            "CREATE TABLE moz_cookies (host TEXT, name TEXT, value TEXT, lastAccessed INTEGER);
             INSERT INTO moz_cookies VALUES ('.music.apple.com', 'media-user-token', 'mock_token_123', 1700000000);"
        ).unwrap();
        drop(conn);

        let token = extract_token_from_db_file(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(token, Some("mock_token_123".to_string()));
    }
}
