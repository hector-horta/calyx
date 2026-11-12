use super::*;
use crate::notice::Tone;

fn text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn nothing_to_say_leaves_the_hints() {
    assert!(list_line(&Slot::default(), &Activity::default(), 60).is_none());
}

#[test]
fn a_list_notice_takes_the_border() {
    let slot = Slot {
        notice: Some(Notice::new("Cola vaciada.".to_string(), Tone::Result)),
        ..Slot::default()
    };
    let line = list_line(&slot, &Activity::default(), 40).map(|l| text(&l));
    assert!(line.is_some_and(|t| t.contains("[ok] Cola vaciada.")));
}

#[test]
fn progress_shows_while_loading_and_shortly_after() {
    let mut slot = Slot::default();
    slot.set_status("Sincronizando playlists...".to_string());
    let mut activity = Activity::default();
    activity.start(Job::Playlists);
    let busy = list_line(&slot, &activity, 60).map(|l| text(&l));
    assert!(busy.is_some_and(|t| t.contains("Sincronizando playlists...")));

    activity.finish(Job::Playlists);
    assert!(list_line(&slot, &activity, 60).is_some(), "fresh result");
    slot.status_at = Some(Instant::now() - STATUS_TTL);
    assert!(list_line(&slot, &activity, 60).is_none(), "old result");
}

#[test]
fn audio_work_does_not_keep_a_list_message_up() {
    let mut slot = Slot::default();
    slot.set_status("Playlists: 12".to_string());
    slot.status_at = Some(Instant::now() - STATUS_TTL);
    let mut activity = Activity::default();
    activity.start(Job::Audio);
    assert!(list_line(&slot, &activity, 60).is_none());
}
