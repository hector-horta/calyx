//! Unit tests for `spectrum` (included via `#[path]` from spectrum.rs).

use super::*;

fn signal(len: usize) -> Vec<i16> {
    (0..len).map(|i| ((i as i32 * 37) % 2000 - 1000) as i16).collect()
}

#[test]
fn empty_input_yields_no_frames() {
    assert!(compute_spectrum_frames(&[], 2, 44_100).is_empty());
}

#[test]
fn frame_count_matches_windows() {
    // 1 s of stereo audio in 50 ms windows.
    assert_eq!(compute_spectrum_frames(&vec![0i16; 44_100 * 2], 2, 44_100).len(), 20);
}

#[test]
fn identical_stereo_equals_mono() {
    let mono = signal(44_100);
    let stereo: Vec<i16> = mono.iter().flat_map(|&s| [s, s]).collect();
    assert_eq!(
        compute_spectrum_frames(&mono, 1, 44_100),
        compute_spectrum_frames(&stereo, 2, 44_100)
    );
}

#[test]
fn levels_stay_in_range() {
    for row in compute_spectrum_frames(&signal(44_100), 1, 44_100) {
        assert!(row.iter().all(|&v| (1..=MAX_LEVEL).contains(&v)));
    }
}

#[test]
fn loud_and_quiet_use_same_range() {
    let loud = signal(44_100);
    let quiet: Vec<i16> = loud.iter().map(|&s| s / 8).collect();
    let peak = |f: Vec<SpectrumFrame>| f.iter().map(|r| r[0]).max().unwrap();
    assert_eq!(peak(compute_spectrum_frames(&loud, 1, 44_100)), peak(compute_spectrum_frames(&quiet, 1, 44_100)));
}

#[test]
fn falloff_is_gradual() {
    let mut pcm = signal(22_050);
    pcm.extend(std::iter::repeat(0i16).take(22_050));
    let frames = compute_spectrum_frames(&pcm, 1, 44_100);
    for pair in frames.windows(2) {
        for k in 0..SPECTRUM_BANDS {
            assert!(pair[0][k].saturating_sub(pair[1][k]) <= FALL_PER_FRAME);
        }
    }
}