use super::*;

#[test]
fn versions_parse_with_or_without_the_v() {
    assert_eq!(parse_version("v0.1.14"), Some((0, 1, 14)));
    assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_version("v1.2"), None);
    assert_eq!(parse_version("v1.2.3.4"), None);
    assert_eq!(parse_version("v1.2.3-beta"), None);
    assert_eq!(parse_version("latest"), None);
}

#[test]
fn newer_compares_numbers_not_text() {
    assert!(is_newer("v0.1.14", "0.1.13"));
    assert!(is_newer("v0.2.0", "0.1.99"));
    assert!(is_newer("v0.1.10", "0.1.9"));
    assert!(!is_newer("v0.1.13", "0.1.13"));
    assert!(!is_newer("v0.1.12", "0.1.13"));
    assert!(!is_newer("garbage", "0.1.13"));
}

#[test]
fn the_cache_is_used_for_one_day() {
    let cache = Cache {
        checked_at: 1_000,
        latest: "v0.1.14".into(),
    };
    assert_eq!(fresh(&cache, 1_000 + DAY_SECS - 1), Some("v0.1.14".into()));
    assert_eq!(fresh(&cache, 1_000 + DAY_SECS), None);
}

#[test]
fn the_release_url_only_comes_from_a_valid_tag() {
    assert_eq!(
        release_url("v0.1.14").as_deref(),
        Some("https://github.com/hector-horta/calyx/releases/tag/v0.1.14")
    );
    assert_eq!(release_url("v0.1.14&calc"), None);
}
