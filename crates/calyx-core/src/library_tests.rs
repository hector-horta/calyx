use super::*;

#[test]
fn test_parse_library_songs_json() {
    let sample_json = r#"
        {
            "data": [
                {
                    "id": "i.12345",
                    "attributes": {
                        "name": "Song Title",
                        "artistName": "Artist Name",
                        "albumName": "Album Name",
                        "durationInMillis": 180000,
                        "artwork": { "url": "https://example.com/artwork/{w}x{h}.jpg" },
                        "playParams": {
                            "catalogId": "1169918082",
                            "purchasedId": null
                        }
                    }
                }
            ]
        }
    "#;

    let result = parse_library_songs_json(sample_json, "uy").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "1169918082");
    assert_eq!(result[0].title, "Song Title");
    assert_eq!(
        result[0].artwork_url.as_deref(),
        Some("https://example.com/artwork/200x200.jpg")
    );
}

#[test]
fn test_parse_user_playlists_json() {
    let sample_json = r#"{"data": [{"id": "p.V7VY2", "attributes": {"name": "Synthwave Mix", "description": {"standard": "Mis favoritos"}}}]}"#;
    let result = parse_user_playlists_json(sample_json).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "p.V7VY2");
    assert_eq!(result[0].name, "Synthwave Mix");
    assert_eq!(result[0].description, "Mis favoritos");
}

#[test]
fn playlists_carry_can_edit() {
    let body = r#"{"data": [
        {"id": "p.Mine", "attributes": {"name": "Mine", "canEdit": true}},
        {"id": "p.Shared", "attributes": {"name": "Shared", "canEdit": false}},
        {"id": "p.Old", "attributes": {"name": "No flag"}}
    ]}"#;
    let pls = parse_user_playlists_json(body).unwrap();
    let flags: Vec<bool> = pls.iter().map(|p| p.can_edit).collect();
    // Without the attribute we let Apple decide (it answers 403 if not allowed).
    assert_eq!(flags, [true, false, true]);
}

#[test]
fn page_info_reads_count_and_next() {
    let body = r#"{"next": "/v1/me/library/songs?offset=100", "data": [{"id": "a"}, {"id": "b"}]}"#;
    assert_eq!(
        page_info(body).unwrap(),
        (2, Some("/v1/me/library/songs?offset=100".to_string()))
    );
    assert_eq!(page_info(r#"{"data": []}"#).unwrap(), (0, None));
}

#[test]
fn page_info_never_follows_other_hosts() {
    let body = r#"{"next": "https://evil.example/v1/x", "data": [{"id": "a"}]}"#;
    assert_eq!(page_info(body).unwrap(), (1, None));
}

#[test]
fn playlist_ids_are_validated() {
    assert!(valid_playlist_id("p.V7VY2"));
    assert!(!valid_playlist_id("p.x/../../me/storefront"));
    assert!(!valid_playlist_id("p.x?limit=1"));
    assert!(!valid_playlist_id(""));
}
