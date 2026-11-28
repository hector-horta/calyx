use super::*;

#[test]
fn every_pose_is_a_full_block_of_half_block_lines() {
    for pose in 0..POSE_COUNT {
        let lines = pose_lines(pose);
        assert_eq!(lines.len(), HEIGHT / 2);
        assert!(lines.iter().all(|l| l.chars().count() == WIDTH));
    }
    assert_eq!(pose_lines(99), pose_lines(0));
}

#[test]
fn poses_differ_and_have_a_round_head() {
    let all: Vec<_> = (0..POSE_COUNT).map(pose_lines).collect();
    for (i, a) in all.iter().enumerate() {
        assert!(all.iter().skip(i + 1).all(|b| b != a), "pose {i} repeats");
    }
    assert_eq!(all[0][0].trim(), "▄███▄");
}
