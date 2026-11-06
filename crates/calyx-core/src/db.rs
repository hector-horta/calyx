use crate::error::CalyxResult;
use crate::models::Track;
use rusqlite::{Connection, Row, params};
use tracing::debug;

/// Column order shared by every INSERT/SELECT; `row_to_track` reads by index.
const COLUMNS: &str = "id, title, artist, album, duration_ms, storefront, preview_url, \
                       artwork_url, hls_manifest_url, is_lossless, album_id";

pub struct CatalogStore {
    conn: Connection,
}

impl CatalogStore {
    pub fn open_in_memory() -> CalyxResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::configure_database(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_path(path: impl AsRef<std::path::Path>) -> CalyxResult<Self> {
        let path = path.as_ref();
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            let _ = std::fs::create_dir_all(dir);
        }
        let conn = Connection::open(path)?;
        Self::configure_database(&conn)?;
        Ok(Self { conn })
    }

    fn configure_database(conn: &Connection) -> CalyxResult<()> {
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        Self::migrate(conn)
    }

    /// Schema migrations, tracked with `PRAGMA user_version`. Add a new
    /// `if version < N` block for every schema change; never edit old ones.
    fn migrate(conn: &Connection) -> CalyxResult<()> {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < 1 {
            // v1: base schema. Databases from before versioning may lack
            // artwork_url, so add it only when it is missing.
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS tracks (
                 id TEXT PRIMARY KEY,
                 title TEXT NOT NULL,
                 artist TEXT NOT NULL,
                 album TEXT NOT NULL,
                 duration_ms INTEGER NOT NULL,
                 storefront TEXT NOT NULL,
                 preview_url TEXT,
                 artwork_url TEXT,
                 hls_manifest_url TEXT,
                 is_lossless INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
             CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);",
            )?;
            if !Self::has_column(conn, "tracks", "artwork_url")? {
                conn.execute_batch("ALTER TABLE tracks ADD COLUMN artwork_url TEXT;")?;
            }
            conn.execute_batch("PRAGMA user_version = 1;")?;
        }
        if version < 2 {
            // v2: catalog album id (open the album a song belongs to).
            conn.execute_batch(
                "ALTER TABLE tracks ADD COLUMN album_id TEXT;
                 PRAGMA user_version = 2;",
            )?;
        }
        Ok(())
    }

    fn has_column(conn: &Connection, table: &str, column: &str) -> CalyxResult<bool> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for name in names {
            if name? == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn delete_track(&self, id: &str) -> CalyxResult<()> {
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all_tracks(&self) -> CalyxResult<()> {
        self.conn.execute("DELETE FROM tracks", [])?;
        Ok(())
    }

    pub fn upsert_track(&self, track: &Track) -> CalyxResult<()> {
        track.validate()?;
        self.conn.execute(
            &format!(
                "INSERT OR REPLACE INTO tracks ({COLUMNS})
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            ),
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_ms as i64,
                track.storefront,
                &track.preview_url,
                &track.artwork_url,
                &track.hls_manifest_url,
                if track.is_lossless { 1 } else { 0 },
                &track.album_id,
            ],
        )?;
        Ok(())
    }

    fn row_to_track(row: &Row<'_>) -> rusqlite::Result<Track> {
        let duration_i64: i64 = row.get(4)?;
        let lossless_i64: i64 = row.get(9)?;
        Ok(Track {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album: row.get(3)?,
            duration_ms: duration_i64.max(0) as u64,
            storefront: row.get(5)?,
            preview_url: row.get::<_, Option<String>>(6)?,
            artwork_url: row.get::<_, Option<String>>(7)?,
            hls_manifest_url: row.get::<_, Option<String>>(8)?,
            is_lossless: lossless_i64 != 0,
            album_id: row.get::<_, Option<String>>(10)?,
        })
    }

    pub fn search_tracks(&self, query: &str, limit: usize) -> CalyxResult<Vec<Track>> {
        let limit_i64 = limit as i64;
        if query.trim().is_empty() {
            debug!("Listing cached tracks (limit={})", limit);
            // Most recent first: INSERT OR REPLACE re-inserts the row, so a higher
            // rowid always means "searched more recently".
            let mut stmt = self.conn.prepare(&format!(
                "SELECT {COLUMNS} FROM tracks ORDER BY rowid DESC LIMIT ?1"
            ))?;
            let rows = stmt.query_map(params![limit_i64], Self::row_to_track)?;
            let mut results = Vec::new();
            for item in rows {
                results.push(item?);
            }
            return Ok(results);
        }

        let pattern = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM tracks
             WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
             ORDER BY title ASC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![pattern, limit_i64], Self::row_to_track)?;
        let mut results = Vec::new();
        for item in rows {
            results.push(item?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sqlite_store_upsert_and_search() {
        let store = CatalogStore::open_in_memory().expect("in-memory db");
        let track = Track {
            id: "100".to_string(),
            title: "Digital Love".to_string(),
            artist: "Daft Punk".to_string(),
            album: "Discovery".to_string(),
            duration_ms: 301_000,
            storefront: "cl".to_string(),
            preview_url: Some("https://audio-ssl.itunes.apple.com/test.m4a".to_string()),
            artwork_url: None,
            hls_manifest_url: None,
            is_lossless: true,
            album_id: None,
        };
        store.upsert_track(&track).expect("upsert");

        let found = store.search_tracks("daft", 10).expect("search");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Digital Love");
    }

    fn track(id: &str, title: &str) -> Track {
        Track {
            id: id.to_string(),
            title: title.to_string(),
            artist: "Artist".to_string(),
            album: "Album".to_string(),
            duration_ms: 1_000,
            storefront: "us".to_string(),
            preview_url: None,
            artwork_url: None,
            hls_manifest_url: None,
            is_lossless: false,
            album_id: None,
        }
    }

    #[test]
    fn recent_list_is_newest_first() {
        let store = CatalogStore::open_in_memory().unwrap();
        store.upsert_track(&track("1", "Zeta")).unwrap();
        store.upsert_track(&track("2", "Alfa")).unwrap();
        let ids = |s: &CatalogStore| {
            s.search_tracks("", 10)
                .unwrap()
                .into_iter()
                .map(|t| t.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&store), ["2", "1"]);
        store.upsert_track(&track("1", "Zeta")).unwrap();
        assert_eq!(ids(&store), ["1", "2"]);
    }

    #[test]
    fn fresh_database_is_at_schema_v2() {
        let store = CatalogStore::open_in_memory().unwrap();
        let v: i64 = store
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 2);
        assert!(CatalogStore::has_column(&store.conn, "tracks", "artwork_url").unwrap());
        assert!(CatalogStore::has_column(&store.conn, "tracks", "album_id").unwrap());
    }

    #[test]
    fn album_id_round_trips() {
        let store = CatalogStore::open_in_memory().unwrap();
        let mut t = track("5", "Let's Dance");
        t.album_id = Some("1440857781".to_string());
        store.upsert_track(&t).unwrap();
        store.upsert_track(&track("6", "No Album")).unwrap();
        let found = store.search_tracks("", 10).unwrap();
        assert_eq!(found[0].album_id, None);
        assert_eq!(found[1].album_id.as_deref(), Some("1440857781"));
    }

    #[test]
    fn v1_database_gains_album_id_and_keeps_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tracks (id TEXT PRIMARY KEY, title TEXT NOT NULL, artist TEXT NOT NULL,
             album TEXT NOT NULL, duration_ms INTEGER NOT NULL, storefront TEXT NOT NULL,
             preview_url TEXT, artwork_url TEXT, hls_manifest_url TEXT, is_lossless INTEGER NOT NULL);
             INSERT INTO tracks VALUES ('7', 'Old', 'A', 'B', 1000, 'us', NULL, NULL, NULL, 0);
             PRAGMA user_version = 1;",
        )
        .unwrap();
        CatalogStore::migrate(&conn).unwrap();
        let store = CatalogStore { conn };
        let rows = store.search_tracks("", 10).unwrap();
        assert_eq!(
            (rows[0].title.as_str(), rows[0].album_id.as_deref()),
            ("Old", None)
        );
        CatalogStore::migrate(&store.conn).unwrap();
    }

    #[test]
    fn legacy_database_gains_artwork_column_and_keeps_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tracks (id TEXT PRIMARY KEY, title TEXT NOT NULL, artist TEXT NOT NULL,
             album TEXT NOT NULL, duration_ms INTEGER NOT NULL, storefront TEXT NOT NULL,
             preview_url TEXT, hls_manifest_url TEXT, is_lossless INTEGER NOT NULL);
             INSERT INTO tracks VALUES ('7', 'Old', 'A', 'B', 1000, 'us', NULL, NULL, 0);",
        )
        .unwrap();
        CatalogStore::migrate(&conn).unwrap();
        let store = CatalogStore { conn };
        assert!(CatalogStore::has_column(&store.conn, "tracks", "artwork_url").unwrap());
        assert_eq!(store.search_tracks("", 10).unwrap()[0].title, "Old");
        // Running it again is a no-op.
        CatalogStore::migrate(&store.conn).unwrap();
    }

    #[test]
    fn test_empty_query() {
        let store = CatalogStore::open_in_memory().expect("in-memory db");
        let tracks = vec![Track {
            id: "1".to_string(),
            title: "Song A".to_string(),
            artist: "Artist 1".to_string(),
            album: "Album 1".to_string(),
            duration_ms: 90_000,
            storefront: "storefront.com".to_string(),
            preview_url: None,
            artwork_url: None,
            hls_manifest_url: None,
            is_lossless: false,
            album_id: None,
        }];
        for track in tracks {
            store.upsert_track(&track).unwrap();
        }

        let results = store.search_tracks("", 1).expect("empty query");
        assert_eq!(results.len(), 1);
    }
}
