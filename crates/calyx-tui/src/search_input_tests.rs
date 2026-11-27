use super::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn typing_and_backspace() {
    let mut s = String::new();
    assert_eq!(edit(&mut s, key(KeyCode::Char('a'))), SearchEdit::Typing);
    edit(&mut s, key(KeyCode::Char('b')));
    assert_eq!(s, "ab");
    edit(&mut s, key(KeyCode::Backspace));
    assert_eq!(s, "a");
}

#[test]
fn esc_clears_first_then_leaves() {
    let mut s = String::from("daft punk");
    assert_eq!(edit(&mut s, key(KeyCode::Esc)), SearchEdit::Typing);
    assert_eq!(s, "");
    assert_eq!(edit(&mut s, key(KeyCode::Esc)), SearchEdit::Leave);
}

#[test]
fn ctrl_u_clears_without_typing_u() {
    let mut s = String::from("ac/dc");
    assert_eq!(edit(&mut s, ctrl('u')), SearchEdit::Typing);
    assert_eq!(s, "");
}

#[test]
fn slash_is_text() {
    let mut s = String::from("ac");
    edit(&mut s, key(KeyCode::Char('/')));
    assert_eq!(s, "ac/");
}

#[test]
fn altgr_characters_are_text() {
    let mut s = String::new();
    let altgr = KeyEvent::new(
        KeyCode::Char('@'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );
    edit(&mut s, altgr);
    assert_eq!(s, "@");
}

#[test]
fn other_ctrl_keys_are_ignored() {
    let mut s = String::from("x");
    edit(&mut s, ctrl('a'));
    assert_eq!(s, "x");
}

#[test]
fn enter_submits() {
    let mut s = String::from("yelle");
    assert_eq!(edit(&mut s, key(KeyCode::Enter)), SearchEdit::Submit);
    assert_eq!(s, "yelle");
}
