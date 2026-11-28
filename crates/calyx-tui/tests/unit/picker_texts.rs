use super::*;

#[test]
fn texts_follow_the_language() {
    assert_eq!(
        picker_texts(Language::Es, "X").new_row,
        "➕ Nueva playlist…"
    );
    assert_eq!(
        picker_texts(Language::En, "X").new_title,
        " New playlist for 'X' "
    );
}
