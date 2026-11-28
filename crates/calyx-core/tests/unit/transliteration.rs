
use super::*;

#[test]
fn test_korean_hangul() {
    assert_eq!(
        transliterate("안녕하세요"),
        Some("annyeonghaseyo".to_string())
    );
}

#[test]
fn test_japanese_kana() {
    assert_eq!(transliterate("あした"), Some("ashita".to_string()));
}

#[test]
fn test_japanese_kanji_and_kana() {
    let res = transliterate("甘くぬかるんだ眼差し").unwrap();
    assert_eq!(res, "amaku nukarunda manazashi");
}

#[test]
fn test_ascii_returns_none() {
    assert_eq!(transliterate("Hello, world!"), None);
}
