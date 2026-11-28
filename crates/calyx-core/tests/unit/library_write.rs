use super::*;

#[test]
fn url_carries_a_valid_album_id() {
    assert_eq!(
        add_album_url("1475002920").unwrap(),
        "https://amp-api.music.apple.com/v1/me/library?ids[albums]=1475002920"
    );
}

#[test]
fn url_rejects_ids_that_could_change_the_query() {
    assert!(add_album_url("1&ids[songs]=2").is_err());
    assert!(add_album_url("").is_err());
    assert!(add_album_url("l.abc").is_err());
}

#[test]
fn status_mapping() {
    assert!(check_status(StatusCode::ACCEPTED).is_ok());
    assert!(matches!(
        check_status(StatusCode::UNAUTHORIZED),
        Err(CalyxError::Auth(_))
    ));
    assert!(matches!(
        check_status(StatusCode::FORBIDDEN),
        Err(CalyxError::Auth(_))
    ));
    assert!(matches!(
        check_status(StatusCode::INTERNAL_SERVER_ERROR),
        Err(CalyxError::Validation(_))
    ));
}

#[test]
fn playlist_request_uses_the_song_kind() {
    let (url, body) = add_to_playlist_request("p.V7VY2", "1475002920").unwrap();
    assert_eq!(
        url,
        "https://amp-api.music.apple.com/v1/me/library/playlists/p.V7VY2/tracks"
    );
    assert_eq!(
        body,
        serde_json::json!({"data": [{"id": "1475002920", "type": "songs"}]})
    );
    let (_, body) = add_to_playlist_request("p.V7VY2", "i.AbC123").unwrap();
    assert_eq!(body["data"][0]["type"], "library-songs");
}

#[test]
fn playlist_request_rejects_bad_ids() {
    assert!(add_to_playlist_request("p.x/../../me", "1").is_err());
    assert!(add_to_playlist_request("p.ok", "1?x=2").is_err());
    assert!(add_to_playlist_request("p.ok", "i.").is_err());
    assert!(add_to_playlist_request("p.ok", "").is_err());
}

#[tokio::test]
async fn needs_a_session_before_any_request() {
    let res = add_album_to_library(&Client::new(), "", "", "1475002920").await;
    assert!(matches!(res, Err(CalyxError::Auth(_))));
}
