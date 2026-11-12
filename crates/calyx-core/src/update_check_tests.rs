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

fn cache(latest: &str) -> Cache {
    Cache {
        checked_at: 1_000,
        latest: latest.into(),
        etag: Some("W/\"abc\"".into()),
    }
}

#[test]
fn a_new_answer_is_used_and_cached_with_its_etag() {
    let answer = Answer::Latest {
        tag: "v0.1.20".into(),
        etag: Some("W/\"new\"".into()),
    };
    let (latest, saved) = resolve(Some(cache("v0.1.19")), Some(answer), 2_000);
    assert_eq!(latest.as_deref(), Some("v0.1.20"));
    let saved = saved.unwrap();
    assert_eq!(
        (saved.checked_at, saved.latest.as_str()),
        (2_000, "v0.1.20")
    );
    assert_eq!(saved.etag.as_deref(), Some("W/\"new\""));
}

#[test]
fn not_modified_or_offline_keeps_the_cached_tag() {
    let (latest, saved) = resolve(Some(cache("v0.1.19")), Some(Answer::Unchanged), 2_000);
    assert_eq!((latest.as_deref(), saved), (Some("v0.1.19"), None));
    let (latest, saved) = resolve(Some(cache("v0.1.19")), None, 2_000);
    assert_eq!((latest.as_deref(), saved), (Some("v0.1.19"), None));
    assert_eq!(resolve(None, None, 2_000), (None, None));
}

#[test]
fn caches_from_before_the_etag_still_load() {
    let old: Cache = serde_json::from_str(r#"{"checked_at":5,"latest":"v0.1.18"}"#).unwrap();
    assert_eq!((old.latest.as_str(), old.etag), ("v0.1.18", None));
}

#[test]
fn the_release_url_only_comes_from_a_valid_tag() {
    assert_eq!(
        release_url("v0.1.14").as_deref(),
        Some("https://github.com/hector-horta/calyx/releases/tag/v0.1.14")
    );
    assert_eq!(release_url("v0.1.14&calc"), None);
}
