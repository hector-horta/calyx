
use super::*;

#[test]
fn every_pose_line_has_the_same_width() {
    for pose in 0..crate::dancer_figure::POSE_COUNT {
        for row in crate::dancer_figure::pose_lines(pose) {
            assert_eq!(Span::raw(row.as_str()).width(), DANCER_WIDTH as usize);
        }
    }
    assert_eq!(Span::raw(notes(0, true)).width(), DANCER_WIDTH as usize);
    assert_eq!(Span::raw(notes(1, true)).width(), DANCER_WIDTH as usize);
}

#[test]
fn stands_still_when_not_playing() {
    assert_eq!(pose_index(24, 7, false), 0);
    assert_eq!(notes(3, false).trim(), "");
}

#[test]
fn loudest_bass_jumps() {
    assert_eq!(pose_index(JUMP_LEVEL, 0, true), 5);
    assert_eq!(pose_index(JUMP_LEVEL, 1, true), 5);
}

#[test]
fn strong_bass_alternates_arm_poses() {
    assert_eq!(pose_index(ARMS_LEVEL, 0, true), 3);
    assert_eq!(pose_index(ARMS_LEVEL, 1, true), 4);
}

#[test]
fn soft_bass_sways() {
    assert_eq!(pose_index(5, 0, true), 1);
    assert_eq!(pose_index(5, 1, true), 2);
}

#[test]
fn block_height_matches_constant() {
    assert_eq!(dancer_lines(10, 0, true, "x").len(), DANCER_HEIGHT as usize);
}
