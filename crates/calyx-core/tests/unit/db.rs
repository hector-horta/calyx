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
