
use super::*;

#[test]
fn frames_advance_and_wrap() {
    assert_eq!(frame_at(0), "⣾");
    assert_eq!(frame_at(99), "⣾");
    assert_eq!(frame_at(100), "⣽");
    assert_eq!(frame_at(800), "⣾");
}

#[test]
fn only_the_cached_track_spins() {
    let caching: Caching = Some(("42".to_string(), Instant::now()));
    assert!(FRAMES.contains(&cache_cell(&caching, "42")));
    assert_eq!(cache_cell(&caching, "7"), BLANK);
    assert_eq!(cache_cell(&None, "42"), BLANK);
}
