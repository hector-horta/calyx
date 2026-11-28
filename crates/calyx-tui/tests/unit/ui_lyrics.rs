use super::*;

#[test]
fn test_wrap_text_long_line() {
    let text = "And it's stickin' out, yeah, like a sore thumb (blowin' up)";
    let wrapped = wrap_text(text, 28);
    assert!(wrapped.len() >= 2);
    assert_eq!(wrapped[0], "And it's stickin' out, yeah,");
}
