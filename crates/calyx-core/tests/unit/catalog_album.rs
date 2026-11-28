use super::*;

#[test]
fn parses_name_and_skips_videos() {
    let body = json!({"data": [{"id": "1440857781", "attributes": {"name": "Let's Dance"},
    "relationships": {"tracks": {"data": [
        {"id": "1440857795", "type": "songs", "attributes": {"name": "Modern Love",
            "artistName": "David Bowie", "albumName": "Let's Dance", "durationInMillis": 286000,
            "url": "https://music.apple.com/us/album/lets-dance/1440857781?i=1440857795"}},
        {"id": "999", "type": "music-videos", "attributes": {"name": "Video"}}
    ]}}}]});
    let (name, tracks) = parse_album_json(&body, "us").unwrap();
    assert_eq!(name, "Let's Dance");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].id, "1440857795");
    assert_eq!(tracks[0].album_id.as_deref(), Some("1440857781"));
}

#[test]
fn missing_album_is_an_error() {
    assert!(parse_album_json(&json!({"data": []}), "us").is_err());
}

#[tokio::test]
async fn library_only_track_without_album_is_rejected_offline() {
    let client = AppleMusicClient::new("us").unwrap();
    let mut t = crate::models::Track {
        id: "i.abc".to_string(),
        title: "Upload".to_string(),
        artist: String::new(),
        album: String::new(),
        duration_ms: 0,
        storefront: "us".to_string(),
        preview_url: None,
        artwork_url: None,
        hls_manifest_url: None,
        is_lossless: false,
        album_id: None,
    };
    assert!(resolve_album_id(&client, &t).await.is_err());
    t.album_id = Some("1440857781".to_string());
    assert_eq!(resolve_album_id(&client, &t).await.unwrap(), "1440857781");
}
