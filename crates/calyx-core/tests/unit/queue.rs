use super::*;

#[test]
fn random_index_reaches_every_position() {
    // Lengths that divide 100 always gave 0 with the old clock-based pick.
    for len in [2usize, 4, 10] {
        let mut seen = vec![false; len];
        for _ in 0..2_000 {
            seen[random_index(len)] = true;
        }
        assert!(seen.iter().all(|s| *s), "len {len}: {seen:?}");
    }
    assert_eq!(random_index(0), 0);
}

fn dummy_track(id: &str) -> Track {
    Track {
        id: id.to_string(),
        title: format!("Title {id}"),
        artist: format!("Artist {id}"),
        album: format!("Album {id}"),
        duration_ms: 180_000,
        storefront: "us".to_string(),
        preview_url: None,
        artwork_url: None,
        hls_manifest_url: None,
        is_lossless: false,
        album_id: None,
    }
}

#[test]
fn test_queue_ordering() {
    let mut q = PlaybackQueue::new();
    q.enqueue_back(dummy_track("1"));
    q.enqueue_back(dummy_track("2"));
    q.enqueue_next(dummy_track("0"));

    assert_eq!(q.len(), 3);
    assert_eq!(q.peek_next().unwrap().id, "0");
    assert_eq!(q.next_track(None).unwrap().id, "0");
    assert_eq!(q.next_track(None).unwrap().id, "1");
    assert_eq!(q.next_track(None).unwrap().id, "2");
    assert!(q.is_empty());
}

#[test]
fn test_repeat_modes() {
    let mut q = PlaybackQueue::new();
    assert_eq!(q.repeat_mode(), RepeatMode::Off);
    q.toggle_repeat();
    assert_eq!(q.repeat_mode(), RepeatMode::All);
    q.toggle_repeat();
    assert_eq!(q.repeat_mode(), RepeatMode::One);

    let cur = dummy_track("current");
    let next = q.next_track(Some(cur.clone())).unwrap();
    assert_eq!(next.id, "current");
}

#[test]
fn previous_without_history_does_not_duplicate() {
    let mut q = PlaybackQueue::new();
    q.enqueue_back(dummy_track("1"));
    assert!(q.previous_track(Some(dummy_track("cur"))).is_none());
    assert_eq!(q.len(), 1);
}

#[test]
fn previous_returns_last_played() {
    let mut q = PlaybackQueue::new();
    q.enqueue_back(dummy_track("1"));
    q.enqueue_back(dummy_track("2"));
    let t1 = q.next_track(None).unwrap();
    let t2 = q.next_track(Some(t1)).unwrap();
    assert_eq!(q.previous_track(Some(t2)).unwrap().id, "1");
    assert_eq!(q.peek_next().unwrap().id, "2");
}

#[test]
fn skip_ignores_repeat_one() {
    let mut q = PlaybackQueue::new();
    q.toggle_repeat();
    q.toggle_repeat();
    q.enqueue_back(dummy_track("2"));
    assert_eq!(q.skip_track(Some(dummy_track("1"))).unwrap().id, "2");
}

#[test]
fn test_repeat_all_cycle() {
    let mut q = PlaybackQueue::new();
    q.toggle_repeat(); // All
    q.enqueue_back(dummy_track("1"));

    let t1 = q.next_track(None).unwrap();
    assert!(q.is_empty());

    let t2 = q.next_track(Some(t1)).unwrap();
    assert_eq!(t2.id, "1");
}
