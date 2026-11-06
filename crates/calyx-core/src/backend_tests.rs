use super::*;

fn backend() -> AppleMusicBackend {
    let session = SessionConfig {
        storefront: "us".to_string(),
        ..Default::default()
    };
    AppleMusicBackend::new(
        session,
        AudioCache::with_dir(std::env::temp_dir().join("calyx_backend_test"), 1024),
    )
    .unwrap()
}

#[test]
fn test_apple_music_backend_metadata() {
    let backend = backend();
    assert_eq!(backend.service_name(), "Apple Music");
    assert!(!backend.is_logged_in());
}

#[test]
fn new_releases_query_uses_a_plausible_year() {
    let q = new_releases_query();
    let year: u64 = q.trim_start_matches("top hits ").parse().unwrap();
    assert!((2024..2100).contains(&year), "{q}");
}

#[test]
fn backend_is_shareable_across_tasks() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<AppleMusicBackend>();
}
