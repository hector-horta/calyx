//! The dancer as pixel art: a chubby figure built from parts (head and
//! belly, each arm up or down, the legs) and drawn with half blocks, two
//! pixels per terminal row, like the big track title.

/// Pixel grid: `WIDTH` columns by `HEIGHT` rows (`HEIGHT / 2` text lines).
pub const WIDTH: usize = 11;
pub const HEIGHT: usize = 14;

type Part = &'static [(usize, usize)];

#[rustfmt::skip]
const HEAD_AND_BELLY: Part = &[
    (4, 0), (5, 0), (6, 0),
    (3, 1), (4, 1), (5, 1), (6, 1), (7, 1),
    (3, 2), (4, 2), (5, 2), (6, 2), (7, 2),
    (4, 3), (5, 3), (6, 3),
    (3, 4), (4, 4), (5, 4), (6, 4), (7, 4),
    (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5),
    (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), (8, 6),
    (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7), (8, 7),
    (3, 8), (4, 8), (5, 8), (6, 8), (7, 8),
    (4, 9), (5, 9), (6, 9),
];
const LEFT_UP: Part = &[(0, 0), (0, 1), (1, 1), (1, 2), (1, 3), (2, 3)];
const LEFT_DOWN: Part = &[(1, 4), (2, 4), (0, 5), (1, 5), (0, 6), (0, 7)];
const RIGHT_UP: Part = &[(10, 0), (9, 1), (10, 1), (9, 2), (8, 3), (9, 3)];
const RIGHT_DOWN: Part = &[(8, 4), (9, 4), (9, 5), (10, 5), (10, 6), (10, 7)];
#[rustfmt::skip]
const LEGS_STAND: Part = &[
    (3, 10), (4, 10), (6, 10), (7, 10),
    (3, 11), (4, 11), (6, 11), (7, 11),
    (2, 12), (3, 12), (7, 12), (8, 12),
    (2, 13), (3, 13), (7, 13), (8, 13),
];
#[rustfmt::skip]
const LEGS_KICK_RIGHT: Part = &[
    (3, 10), (4, 10), (6, 10), (7, 10),
    (3, 11), (4, 11), (7, 11), (8, 11),
    (2, 12), (3, 12), (8, 12), (9, 12),
    (2, 13), (3, 13), (9, 13), (10, 13),
];
#[rustfmt::skip]
const LEGS_KICK_LEFT: Part = &[
    (3, 10), (4, 10), (6, 10), (7, 10),
    (2, 11), (3, 11), (6, 11), (7, 11),
    (1, 12), (2, 12), (7, 12), (8, 12),
    (0, 13), (1, 13), (7, 13), (8, 13),
];
#[rustfmt::skip]
const LEGS_JUMP: Part = &[
    (2, 10), (3, 10), (7, 10), (8, 10),
    (1, 11), (2, 11), (8, 11), (9, 11),
];

/// Parts of each pose: 0 idle, 1-2 sway, 3-4 arms up, 5 jump.
const POSES: [[Part; 3]; 6] = [
    [LEFT_DOWN, RIGHT_DOWN, LEGS_STAND],
    [LEFT_UP, RIGHT_DOWN, LEGS_KICK_RIGHT],
    [LEFT_DOWN, RIGHT_UP, LEGS_KICK_LEFT],
    [LEFT_UP, RIGHT_UP, LEGS_STAND],
    [LEFT_UP, RIGHT_UP, LEGS_KICK_RIGHT],
    [LEFT_UP, RIGHT_UP, LEGS_JUMP],
];

#[cfg(test)]
pub const POSE_COUNT: usize = POSES.len();

/// Text lines of `pose` (`HEIGHT / 2` of them, `WIDTH` columns each).
pub fn pose_lines(pose: usize) -> Vec<String> {
    let mut grid = [[false; WIDTH]; HEIGHT];
    let parts = POSES.get(pose).unwrap_or(&POSES[0]);
    for &(x, y) in HEAD_AND_BELLY
        .iter()
        .chain(parts.iter().flat_map(|p| p.iter()))
    {
        if let Some(cell) = grid.get_mut(y).and_then(|row| row.get_mut(x)) {
            *cell = true;
        }
    }
    grid.chunks(2)
        .map(|pair| {
            let top = &pair[0];
            let bottom = &pair[1];
            (0..WIDTH)
                .map(|x| match (top[x], bottom[x]) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                })
                .collect::<String>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
}
