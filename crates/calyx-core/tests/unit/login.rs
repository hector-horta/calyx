use super::*;

#[test]
fn test_extract_token_from_cdp_json() {
    let cookies_json = r#"{"id":1,"result":{"cookies":[{"name":"itua","value":"cl"},{"name":"media-user-token","value":"Aq123456789012345678901234567890==","domain":".music.apple.com"}]}}"#;
    assert_eq!(
        extract_token_from_cdp_json(cookies_json).as_deref(),
        Some("Aq123456789012345678901234567890==")
    );

    let eval_json = r#"{"id":2,"result":{"result":{"type":"string","value":"MusicKitUserToken98765432109876543210"}}}"#;
    assert_eq!(
        extract_token_from_cdp_json(eval_json).as_deref(),
        Some("MusicKitUserToken98765432109876543210")
    );
}

#[test]
fn test_extract_tokens_from_cdp_eval() {
    let raw = r#"{"id":3,"result":{"result":{"type":"string","value":"{\"dev\":\"eyJhbGciOiJFUzI1NiIsabcdefghijklmnopqrstuvwxyz01234567890123456789\",\"user\":\"MusicKitUserToken98765432109876543210\"}"}}}"#;
    let (dev, user) = extract_tokens_from_cdp_eval(raw).unwrap();
    assert!(dev.unwrap().starts_with("eyJ"));
    assert_eq!(user, "MusicKitUserToken98765432109876543210");
}
