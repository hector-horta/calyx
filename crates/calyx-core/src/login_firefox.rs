use crate::browser_discovery::find_gecko_cookie_databases;
use crate::error::{CalyxError, CalyxResult};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

pub fn extract_token_from_db_file(db_path: &Path) -> Option<String> {
    // One snapshot per call: two readers must not share (and delete) a copy.
    static SNAPSHOTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SNAPSHOTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp_path =
        std::env::temp_dir().join(format!("calyx_ff_snap_{}_{n}.sqlite", std::process::id()));
    if std::fs::copy(db_path, &tmp_path).is_err() {
        return None;
    }
    // Firefox keeps recent cookies (a login from seconds ago) in the WAL
    // until its next checkpoint, so the snapshot needs it too.
    let side = |path: &Path, suffix: &str| PathBuf::from(format!("{}{suffix}", path.display()));
    let tmp_wal = side(&tmp_path, "-wal");
    let _ = std::fs::remove_file(&tmp_wal);
    let _ = std::fs::copy(side(db_path, "-wal"), &tmp_wal);

    let res = (|| {
        let conn =
            Connection::open_with_flags(&tmp_path, OpenFlags::SQLITE_OPEN_READ_WRITE).ok()?;
        let mut stmt = conn
            .prepare(
                "SELECT value FROM moz_cookies 
             WHERE (host LIKE '%apple.com' OR host LIKE '%mzstatic.com') 
               AND name = 'media-user-token' 
             ORDER BY lastAccessed DESC LIMIT 1",
            )
            .ok()?;
        let mut rows = stmt.query([]).ok()?;
        if let Some(row) = rows.next().ok().flatten() {
            let val: String = row.get(0).ok()?;
            if !val.trim().is_empty() {
                return Some(val);
            }
        }
        None
    })();

    for leftover in [tmp_path.clone(), tmp_wal, side(&tmp_path, "-shm")] {
        let _ = std::fs::remove_file(leftover);
    }
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
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on macOS: {e}")))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on Windows: {e}")))?;

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| CalyxError::Auth(format!("Failed to open browser on Linux: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_mock_moz_cookies() {
        let tmp =
            std::env::temp_dir().join(format!("test_moz_cookies_{}.sqlite", std::process::id()));
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

    #[test]
    fn a_cookie_still_in_the_wal_is_found() {
        let tmp = std::env::temp_dir().join(format!("test_moz_wal_{}.sqlite", std::process::id()));
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", tmp.display()));
        }
        // The writer stays open and never checkpoints, like a running Firefox.
        let conn = Connection::open(&tmp).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;
             CREATE TABLE moz_cookies (host TEXT, name TEXT, value TEXT, lastAccessed INTEGER);
             INSERT INTO moz_cookies VALUES ('.music.apple.com', 'media-user-token', 'wal_token', 1);",
        )
        .unwrap();

        let token = extract_token_from_db_file(&tmp);
        drop(conn);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", tmp.display()));
        }
        assert_eq!(token.as_deref(), Some("wal_token"));
    }
}
