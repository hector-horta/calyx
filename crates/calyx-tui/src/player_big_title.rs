//! The song title in big quadrant letters for the player's song box. A title
//! wider than the box scrolls past once when the song starts, then rests on
//! its beginning, so it does not compete with the spectrum all song long.

use crate::app::TuiApp;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Widget,
};
use tui_big_text::{BigText, PixelSize};

/// Rows and columns of one big (quadrant) letter.
pub const BIG_H: u16 = 4;
const BIG_COL: u16 = 4;
/// Columns of the "..." drawn after a cut title (three dots, one apart).
const ELLIPSIS_W: u16 = 5;
const STYLE: Style = Style::new().fg(Color::White).add_modifier(Modifier::BOLD);
/// Song time shown still before the scroll starts.
const HOLD_MS: u64 = 2000;
/// Song time per scrolled column.
const STEP_MS: u64 = 90;
/// Song time shown at the end of the title before going back to its start.
const END_HOLD_MS: u64 = 1500;

pub struct BigTitle {
    text: BigText<'static>,
    /// Columns of the whole title.
    width: u16,
    /// First column shown.
    offset: u16,
    /// Which letters are spaces (an ellipsis never follows one).
    spaces: Vec<bool>,
}

impl BigTitle {
    /// Draws the visible window of the title: the whole title is drawn into
    /// its own buffer and the columns from `offset` copied into `area`.
    pub fn render(self, frame: &mut Frame<'_>, area: Rect) {
        // At rest a title too wide for the box ends in "..." after a whole letter.
        let cut = self.offset == 0 && self.width > area.width;
        let shown = if cut {
            self.letters_before_ellipsis(area.width) * BIG_COL
        } else {
            area.width
        };
        let mut own = Buffer::empty(Rect::new(0, 0, self.width, BIG_H));
        self.text.render(own.area, &mut own);
        let buf = frame.buffer_mut();
        for y in 0..area.height.min(BIG_H) {
            for x in 0..shown.min(self.width - self.offset) {
                buf[(area.x + x, area.y + y)] = own[(self.offset + x, y)].clone();
            }
        }
        if cut && area.height >= BIG_H {
            // Three one-column dots on the baseline, like the big '.'.
            for dot in 0..3 {
                let x = area.x + shown + dot * 2;
                buf[(x, area.y + 2)].set_symbol("▄").set_style(STYLE);
                buf[(x, area.y + 3)].set_symbol("▀").set_style(STYLE);
            }
        }
    }

    /// Whole letters that fit before the ellipsis, without trailing spaces.
    fn letters_before_ellipsis(&self, width: u16) -> u16 {
        let mut n = (width.saturating_sub(ELLIPSIS_W) / BIG_COL) as usize;
        while n > 0 && self.spaces.get(n - 1).copied().unwrap_or(false) {
            n -= 1;
        }
        n as u16
    }
}

/// First column to show `elapsed_ms` into the song for a title `overflow`
/// columns wider than the box: still, one pass to the end, then the start.
pub fn scroll_offset(elapsed_ms: u64, overflow: u16) -> u16 {
    let scroll_ms = overflow as u64 * STEP_MS;
    match elapsed_ms.checked_sub(HOLD_MS) {
        Some(t) if t < scroll_ms => (t / STEP_MS) as u16,
        Some(t) if t < scroll_ms + END_HOLD_MS => overflow,
        _ => 0,
    }
}

/// The title in big letters; a " (...)", " [...]" or " - ..." suffix is
/// dropped when the whole title does not fit ("Maniac (From Flashdance)").
pub fn big_title(app: &TuiApp, width: u16) -> Option<BigTitle> {
    let full = app.current_track.as_ref()?.title.trim();
    let cut = [" (", " [", " - "]
        .iter()
        .filter_map(|sep| full.find(sep))
        .min()
        .unwrap_or(full.len());
    let short = full[..cut].trim_end();
    let cols = |t: &str| t.chars().count() as u16 * BIG_COL;
    let title = if cols(full) <= width { full } else { short };
    // The font only has ASCII: accents are dropped, other scripts keep the
    // plain line.
    let title: String = title.chars().map(plain).collect();
    if !title.is_ascii() {
        return None;
    }
    let title_w = cols(&title);
    let spaces = title.chars().map(|c| c == ' ').collect();
    let elapsed = app.playback_progress().0.as_millis() as u64;
    let offset = scroll_offset(elapsed, title_w.saturating_sub(width));
    Some(BigTitle {
        text: BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(STYLE)
            .lines(vec![Line::from(title)])
            .build(),
        width: title_w,
        offset,
        spaces,
    })
}

/// The letter without its accent ("á" -> 'a', "Ñ" -> 'N'); curly quotes as '.
fn plain(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'Á' | 'À' | 'Ä' | 'Â' | 'Ã' => 'A',
        'É' | 'È' | 'Ë' | 'Ê' => 'E',
        'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
        'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' => 'O',
        'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ç' => 'c',
        'Ç' => 'C',
        '’' | '‘' => '\'',
        other => other,
    }
}

#[cfg(test)]
#[path = "player_big_title_tests.rs"]
mod tests;
