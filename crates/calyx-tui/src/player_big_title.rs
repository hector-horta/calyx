//! The song title in big quadrant letters for the player's song box.

use crate::app::TuiApp;
use ratatui::{
    style::{Color, Style},
    text::Line,
};
use tui_big_text::{BigText, PixelSize};

/// Rows and columns of one big (quadrant) letter.
pub const BIG_H: u16 = 4;
const BIG_COL: u16 = 4;

/// The title in quadrant blocks (4x4 cells per letter), only when it fits.
pub fn big_title(app: &TuiApp, width: u16) -> Option<BigText<'static>> {
    let full = app.current_track.as_ref()?.title.trim();
    // "Maniac (From Flashdance)" / "Song - Remastered": when the whole title
    // does not fit, the big one drops the suffix.
    let cut = [" (", " [", " - "]
        .iter()
        .filter_map(|sep| full.find(sep))
        .min()
        .unwrap_or(full.len());
    let short = &full[..cut];
    let fits = |t: &str| t.chars().count() as u16 * BIG_COL <= width;
    let title = [full, short].into_iter().find(|t| fits(t))?.to_string();
    Some(
        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().fg(Color::White))
            .lines(vec![Line::from(title)])
            .build(),
    )
}
