
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
