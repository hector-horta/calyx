use super::*;

#[test]
fn the_cli_storefront_wins_unless_logged_in() {
    let mut cfg = SessionConfig {
        storefront: "us".into(),
        ..Default::default()
    };
    cfg.apply_cli_storefront(None);
    assert_eq!(cfg.storefront, "us");
    cfg.apply_cli_storefront(Some("mx"));
    assert_eq!(cfg.storefront, "mx");

    cfg.media_user_token = Some("token".into());
    cfg.apply_cli_storefront(Some("jp"));
    assert_eq!(cfg.storefront, "mx", "the account's storefront is kept");
}

#[test]
fn test_extract_js_and_jwt() {
    let sample_html =
        r#"<script type="module" crossorigin src="/assets/index-legacy-a1b2c3.js"></script>"#;
    assert_eq!(
        extract_js_bundle_path(sample_html),
        Some("/assets/index-legacy-a1b2c3.js".to_string())
    );

    let fake_jwt = format!(
        "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.{}.{}",
        "a".repeat(80),
        "b".repeat(60)
    );
    let bundle = format!("const devToken = \"{}\";", fake_jwt);
    assert_eq!(extract_jwt_from_bundle(&bundle), Some(fake_jwt));
}
