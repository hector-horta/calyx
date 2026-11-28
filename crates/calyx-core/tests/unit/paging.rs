use super::*;

#[test]
fn page_info_reads_count_and_next() {
    let body = r#"{"next": "/v1/me/library/songs?offset=100", "data": [{"id": "a"}, {"id": "b"}]}"#;
    assert_eq!(
        page_info(body).unwrap(),
        (2, Some("/v1/me/library/songs?offset=100".to_string()))
    );
    assert_eq!(page_info(r#"{"data": []}"#).unwrap(), (0, None));
}

#[test]
fn page_total_comes_from_meta() {
    assert_eq!(
        page_total(r#"{"meta": {"total": 1234}, "data": []}"#),
        Some(1234)
    );
    assert_eq!(page_total(r#"{"data": []}"#), None);
    assert_eq!(page_total("not json"), None);
}

#[test]
fn page_info_never_follows_other_hosts() {
    let body = r#"{"next": "https://evil.example/v1/x", "data": [{"id": "a"}]}"#;
    assert_eq!(page_info(body).unwrap(), (1, None));
}
