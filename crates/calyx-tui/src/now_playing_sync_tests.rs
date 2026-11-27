use super::*;

fn song(id: &str, state: PlayState) -> Shown {
    (id.to_string(), state)
}

fn ago(secs: u64) -> Option<Instant> {
    Instant::now().checked_sub(Duration::from_secs(secs))
}

#[test]
fn a_new_song_publishes_everything() {
    assert_eq!(
        due(None, Some(&song("1", PlayState::Playing)), None, None),
        Due {
            new_song: true,
            os: true,
            file: true,
            clear: false
        }
    );
}

#[test]
fn nothing_to_do_right_after_publishing() {
    let s = song("1", PlayState::Paused);
    assert_eq!(due(Some(&s), Some(&s), ago(0), ago(0)), Due::default());
}

#[test]
fn playing_refreshes_the_position_every_second() {
    let s = song("1", PlayState::Playing);
    let d = due(Some(&s), Some(&s), ago(2), ago(0));
    assert!(d.os);
    assert!(!d.new_song);
    assert!(!d.file);
}

#[test]
fn pausing_is_not_a_new_song() {
    let d = due(
        Some(&song("1", PlayState::Playing)),
        Some(&song("1", PlayState::Paused)),
        ago(0),
        ago(0),
    );
    assert!(d.os && d.file && !d.new_song);
}

#[test]
fn the_file_is_refreshed_as_a_heartbeat() {
    let s = song("1", PlayState::Paused);
    let d = due(Some(&s), Some(&s), ago(0), ago(20));
    assert!(d.file && !d.os);
}

#[test]
fn stopping_clears_once() {
    assert!(due(Some(&song("1", PlayState::Playing)), None, None, None).clear);

    assert_eq!(due(None, None, None, None), Due::default());
}
