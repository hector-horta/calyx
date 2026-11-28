
use super::*;

fn t(title: &str, artist: &str) -> Track {
    Track {
        id: "1".to_string(),
        title: title.to_string(),
        artist: artist.to_string(),
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
fn same_song_ignores_suffixes() {
    assert!(
        t("Digital Love (Remastered)", "Daft Punk").is_same_song(&t("Digital Love", "Daft Punk"))
    );
}

#[test]
fn different_title_is_not_same() {
    assert!(!t("One More Time", "Daft Punk").is_same_song(&t("Digital Love", "Daft Punk")));
}

#[test]
fn featured_artist_still_matches() {
    assert!(
        t("Get Lucky", "Daft Punk").is_same_song(&t("Get Lucky", "Daft Punk, Pharrell Williams"))
    );
}

#[test]
fn different_artist_is_not_same() {
    assert!(!t("Hurt", "Nine Inch Nails").is_same_song(&t("Hurt", "Johnny Cash")));
}

#[test]
fn non_ascii_titles_do_not_panic() {
    assert!(t("夜に駆ける - Remix", "YOASOBI").is_same_song(&t("夜に駆ける", "YOASOBI")));
    assert!(t("Beyoncé (Live)", "Beyoncé").is_same_song(&t("Beyoncé", "Beyoncé")));
}
