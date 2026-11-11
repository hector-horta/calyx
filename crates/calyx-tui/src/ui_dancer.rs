//! ASCII dancer shown in the lyrics pane when a track has no lyrics.
//!
//! Stateless: the pose comes from the playback clock plus the bass level of
//! the precomputed spectrum, so bigger moves land on the louder bass hits.

use crate::app::TuiApp;
use calyx_core::models::PlaybackStatus;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Columns of every pose line.
pub const DANCER_WIDTH: u16 = crate::dancer_figure::WIDTH as u16;
/// Notes line + the figure + blank + caption.
pub const DANCER_HEIGHT: u16 = crate::dancer_figure::HEIGHT as u16 / 2 + 3;
/// One dance step every this many milliseconds.
const STEP_MS: u64 = 400;
/// Bass level (1..=24) from which the arms go up.
const ARMS_LEVEL: u8 = 19;
/// Bass level from which the dancer jumps (the loudest hits).
const JUMP_LEVEL: u8 = 24;

const FIGURE: Style = Style::new().fg(Color::Rgb(0, 240, 180));
const NOTES: Style = Style::new().fg(Color::Rgb(255, 215, 0));
const CAPTION: Style = Style::new().fg(Color::DarkGray);

/// Picks the pose for this moment: idle when not playing; otherwise sway,
/// raise the arms or jump depending on how strong the bass is.
pub fn pose_index(bass: u8, step: u64, playing: bool) -> usize {
    if !playing {
        return 0;
    }
    let alt = (step % 2) as usize;
    if bass >= JUMP_LEVEL {
        5
    } else if bass >= ARMS_LEVEL {
        3 + alt
    } else {
        1 + alt
    }
}

fn notes(step: u64, playing: bool) -> &'static str {
    match (playing, step % 2) {
        (false, _) => "           ",
        (true, 0) => "♪         ♫",
        (true, _) => "   ♫   ♪   ",
    }
}

/// The dancer block: floating notes, the figure and a caption underneath.
pub fn dancer_lines(
    bass: u8,
    elapsed_ms: u64,
    playing: bool,
    caption: &'static str,
) -> Vec<Line<'static>> {
    let step = elapsed_ms / STEP_MS;
    let pose = pose_index(bass, step, playing);
    let mut lines = Vec::with_capacity(DANCER_HEIGHT as usize);
    lines.push(Line::from(Span::styled(notes(step, playing), NOTES)));
    lines.extend(
        crate::dancer_figure::pose_lines(pose)
            .into_iter()
            .map(|row| Line::from(Span::styled(row, FIGURE))),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(caption, CAPTION)));
    lines
}

/// Whether the dancer fits in the lyrics pane (it has a left border).
pub fn fits(area: Rect) -> bool {
    area.width >= DANCER_WIDTH + 2 && area.height >= DANCER_HEIGHT
}

/// Draws the dancer centered in the lyrics pane.
pub fn render_dancer(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let playing = matches!(app.playback_status(), PlaybackStatus::Playing);
    let bass = app.current_spectrum()[0];
    let elapsed_ms = app.playback_progress().0.as_millis() as u64;
    let block = Block::default().borders(Borders::LEFT);
    let top = block.inner(area).height.saturating_sub(DANCER_HEIGHT) / 2;
    let mut lines = vec![Line::from(""); top as usize];
    lines.extend(dancer_lines(
        bass,
        elapsed_ms,
        playing,
        app.language.no_lyrics_track(),
    ));
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}
#[cfg(test)]
mod tests {
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
}
