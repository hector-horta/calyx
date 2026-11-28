use super::*;
use crate::messages::Msg;
use crate::messages_picker::PickerMsg;

fn text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn result_line_is_tagged_and_padded_to_width() {
    let n = Notice::new("Cola vaciada".to_string(), Tone::Result);
    assert!(text(&n.line(30)).starts_with(" [ok] Cola vaciada"));
    assert_eq!(text(&n.line(30)).chars().count(), 30);
}

#[test]
fn padding_counts_characters_not_bytes() {
    let n = Notice::new("¿Quitar 'Mataz'?".to_string(), Tone::Error);
    assert_eq!(text(&n.line(40)).chars().count(), 40);
}

#[test]
fn action_shows_a_countdown_and_long_text_is_cut() {
    let n = Notice::new("Presiona d otra vez".to_string(), Tone::Action);
    assert!(text(&n.line(60)).contains("(4s)"));
    assert_eq!(text(&n.line(10)).chars().count(), 10);
    assert!(n.is_live());
}

#[test]
fn messages_are_classified() {
    assert_eq!(Msg::QueueCleared.tone(), Some(Tone::Result));
    assert_eq!(Msg::AudioError("x").tone(), Some(Tone::Error));
    assert_eq!(Msg::Searching("x").tone(), None);
    assert_eq!(
        PickerMsg::ConfirmRemove {
            title: "a",
            playlist: "b"
        }
        .tone(),
        Some(Tone::Action)
    );
}

#[test]
fn notices_expire_after_their_tone_ttl() {
    let mut timed = Notice::new("Presiona d otra vez".to_string(), Tone::Action);
    assert!(timed.is_live());
    timed.at -= Duration::from_secs(60);
    assert!(!timed.is_live());
}
