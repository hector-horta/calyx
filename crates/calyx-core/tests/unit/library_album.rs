use super::*;

#[test]
fn album_ref_reads_the_library_album() {
    let body = r#"{"data":[{"id":"l.2OsPz4t","attributes":{"name":"Kitsuné : À cause des garçons (Remixes) - EP"}}]}"#;
    assert_eq!(
        album_ref(body),
        Some((
            "l.2OsPz4t".to_string(),
            "Kitsuné : À cause des garçons (Remixes) - EP".to_string()
        ))
    );
}

#[test]
fn album_ref_rejects_ids_that_could_change_the_url() {
    for id in ["l.", "l.a/../b", "1440857781", "l.a?x=1"] {
        let body = format!(r#"{{"data":[{{"id":"{id}","attributes":{{"name":"x"}}}}]}}"#);
        assert_eq!(album_ref(&body), None, "{id}");
    }
    assert_eq!(album_ref(r#"{"data":[]}"#), None);
    assert_eq!(album_ref("not json"), None);
}

#[tokio::test]
async fn needs_a_session_and_a_library_song_id() {
    let http = Client::new();
    assert!(matches!(
        fetch_library_album_of(&http, "", "", "i.abc", "us").await,
        Err(CalyxError::Auth(_))
    ));
    assert!(matches!(
        fetch_library_album_of(&http, "dev", "usr", "468319426", "us").await,
        Err(CalyxError::Validation(_))
    ));
}
