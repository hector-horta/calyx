use super::*;

fn track(id: &str, alt: Option<&str>) -> Track {
    Track {
        id: id.to_string(),
        title: "Digital Love (Remastered)".to_string(),
        artist: "Daft Punk".to_string(),
        album: "Discovery".to_string(),
        duration_ms: 1_000,
        storefront: "us".to_string(),
        preview_url: None,
        artwork_url: None,
        hls_manifest_url: alt.map(str::to_string),
        is_lossless: false,
        album_id: None,
    }
}

fn temp_cache(name: &str) -> AudioCache {
    let dir = std::env::temp_dir().join(format!("calyx_resolver_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    AudioCache::with_dir(dir, 10_000)
}

#[test]
fn candidates_skip_duplicate_and_empty_alt_ids() {
    assert_eq!(candidate_ids(&track("1", Some("i.9"))), ["1", "i.9"]);
    assert_eq!(candidate_ids(&track("1", Some("1"))), ["1"]);
    assert_eq!(candidate_ids(&track("1", Some(""))), ["1"]);
}

#[test]
fn search_query_drops_suffixes() {
    assert_eq!(search_query(&track("1", None)), "Daft Punk Digital Love");
}

#[tokio::test]
async fn cached_alt_id_is_served_without_network() {
    let cache = temp_cache("hit");
    cache.put("i.9", b"m4a").unwrap();
    let client = AppleMusicClient::new("us").unwrap();
    let got = resolve_audio(
        &client,
        &SessionConfig::default(),
        &cache,
        &track("1", Some("i.9")),
    )
    .await
    .unwrap();
    assert_eq!(
        (got.source, got.bytes),
        (AudioSource::Cache, b"m4a".to_vec())
    );
}

#[tokio::test]
async fn nothing_playable_is_an_error() {
    let cache = temp_cache("miss");
    let client = AppleMusicClient::new("us").unwrap();
    let err = resolve_audio(
        &client,
        &SessionConfig::default(),
        &cache,
        &track("1", None),
    )
    .await;
    assert!(err.is_err());
}
