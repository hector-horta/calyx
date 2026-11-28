use super::*;

#[test]
fn test_language_toggle() {
    assert_eq!(Language::Es.toggle(), Language::En);
    assert_eq!(Language::En.toggle(), Language::Es);
    assert_eq!(Language::Es.code(), "es");
    assert_eq!(Language::En.code(), "en");
}
