use super::*;

#[test]
fn name_is_trimmed() {
    assert_eq!(clean_playlist_name("  Bowie  ").unwrap(), "Bowie");
}

#[test]
fn bad_names_are_rejected() {
    assert!(clean_playlist_name("").is_err());
    assert!(clean_playlist_name("   ").is_err());
    assert!(clean_playlist_name("a\nb").is_err());
    assert!(clean_playlist_name(&"x".repeat(101)).is_err());

    assert!(clean_playlist_name(&"x".repeat(100)).is_ok());
    assert!(clean_playlist_name("Café 夜").is_ok());
}

#[test]
fn request_creates_with_the_song() {
    let (url, body) = create_playlist_request(" Mix ", "1440857795").unwrap();
    assert_eq!(
        url,
        "https://amp-api.music.apple.com/v1/me/library/playlists"
    );
    assert_eq!(
        body,
        serde_json::json!({"attributes": {"name": "Mix"}, "relationships": {"tracks": {"data": [{"id": "1440857795", "type": "songs"}]}}})
    );
}

#[test]
fn request_uses_library_kind_and_rejects_bad_song() {
    let (_, body) = create_playlist_request("Mix", "i.AbC1").unwrap();
    assert_eq!(
        body["relationships"]["tracks"]["data"][0]["type"],
        "library-songs"
    );
    assert!(create_playlist_request("Mix", "1?x=2").is_err());
}

#[test]
fn parses_the_new_playlist() {
    let p = parse_created_playlist(
        r#"{"data": [{"id": "p.New1", "attributes": {"name": "Mix", "canEdit": true}}]}"#,
    )
    .unwrap();
    assert_eq!(
        (p.id.as_str(), p.name.as_str(), p.can_edit),
        ("p.New1", "Mix", true)
    );
    assert!(parse_created_playlist(r#"{"data": []}"#).is_err());
}

#[tokio::test]
async fn needs_a_session_before_any_request() {
    let res =
        create_playlist_with_track(&reqwest::Client::new(), "", "", "Mix", "1440857795").await;
    assert!(matches!(res, Err(CalyxError::Auth(_))));
}
