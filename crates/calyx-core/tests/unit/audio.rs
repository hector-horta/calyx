
use super::*;

fn headless() -> NativeAudioEngine {
    NativeAudioEngine::with_output(None, None)
}

fn one_second_track() -> DecodedTrack {
    DecodedTrack {
        channels: 1,
        sample_rate: 8_000,
        pcm: vec![0; 8_000],
        spectrum: vec![[7; SPECTRUM_BANDS]; 20],
    }
}

#[test]
fn test_empty_audio_buffer_returns_error() {
    assert!(headless().play_encoded_bytes(Vec::new()).is_err());
}

#[test]
fn pause_freezes_progress_and_resume_continues() {
    let engine = headless();
    engine.play_decoded(one_second_track()).unwrap();
    assert_eq!(engine.status(), PlaybackStatus::Playing);
    assert_eq!(engine.progress().1, Duration::from_secs(1));

    engine.toggle_pause();
    assert_eq!(engine.status(), PlaybackStatus::Paused);
    let frozen = engine.progress().0;
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(engine.progress().0, frozen);

    engine.toggle_pause();
    std::thread::sleep(Duration::from_millis(30));
    assert!(engine.progress().0 > frozen);
}

#[test]
fn stop_silences_spectrum_and_resets_progress() {
    let engine = headless();
    engine.play_decoded(one_second_track()).unwrap();
    assert_eq!(engine.current_spectrum(), [7; SPECTRUM_BANDS]);
    engine.stop();
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert_eq!(engine.current_spectrum(), [0; SPECTRUM_BANDS]);
    assert_eq!(engine.progress().0, Duration::ZERO);
}

#[test]
fn headless_playback_finishes_when_the_time_runs_out() {
    let engine = headless();
    engine.play_decoded(one_second_track()).unwrap();
    assert!(!engine.is_finished());

    // Shorter than a second: its whole-second duration is zero.
    let mut short = one_second_track();
    short.pcm.truncate(100);
    engine.play_decoded(short).unwrap();
    assert!(engine.is_finished());
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert!(!engine.is_finished(), "reported only once");
}

#[test]
fn volume_is_clamped() {
    let engine = headless();
    engine.set_volume(3.0);
    assert_eq!(engine.lock().volume, 1.0);
}
