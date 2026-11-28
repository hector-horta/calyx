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
#[path = "../tests/unit/db.rs"]
mod tests;
