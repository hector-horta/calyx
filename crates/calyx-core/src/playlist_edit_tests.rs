use super::*;

#[test]
fn remove_puts_library_ids_and_mode_in_the_url() {
    let url = remove_tracks_url("p.Trash1", &["i.AbC1", "i.XyZ2"]).unwrap();
    assert_eq!(
        url,
        "https://amp-api.music.apple.com/v1/me/library/playlists/p.Trash1/tracks?ids[library-songs]=i.AbC1,i.XyZ2&mode=all"
    );
}

#[test]
fn bad_ids_never_reach_the_url() {
    assert!(remove_tracks_url("p.x/../me", &["i.A"]).is_err());
    assert!(remove_tracks_url("p.Ok", &[]).is_err());
    assert!(remove_tracks_url("p.Ok", &["1440857795"]).is_err());
    assert!(remove_tracks_url("p.Ok", &["i.A&mode=x"]).is_err());
    assert!(remove_tracks_url("p.Ok", &["i."]).is_err());
}
