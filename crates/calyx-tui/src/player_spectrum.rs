//! Responsive spectrum bars for the player panel.
//!
//! Bars widen to fill the panel; below `NARROW_BELOW` columns the big play icon
//! and the volume meter are dropped so the spectrum is never clipped.

use calyx_core::{SpectrumFrame, SPECTRUM_BANDS};
use ratatui::{
    style::{Color, Style},
    text::Span,
};

const BAR_GLYPHS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
/// Play icon (6) + gap (2) on the left, gap (2) + volume meter (6) on the right.
pub const SIDE_COLS: u16 = 16;
/// Inner widths below this drop the big icon and the volume meter.
pub const NARROW_BELOW: u16 = 60;
/// Inner widths from this up spread the status line to the panel edges.
pub const WIDE_FROM: u16 = 90;
const MAX_BAR_W: u16 = 8;
/// Background the pending (not yet played) bars are mixed towards.
const TRACK_BG: Color = Color::Rgb(40, 44, 52);
/// How much of the bar colour a pending bar keeps.
const PENDING_MIX: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarLayout {
    pub bar_w: u16,
    pub gap: u16,
    pub show_sides: bool,
}

impl BarLayout {
    pub fn for_width(inner: u16) -> Self {
        let show_sides = inner >= NARROW_BELOW;
        let gap = if show_sides { 1 } else { 0 };
        let side = if show_sides { SIDE_COLS } else { 0 };
        let gaps = gap * (SPECTRUM_BANDS as u16 - 1);
        let bar_w = (inner.saturating_sub(side + gaps) / SPECTRUM_BANDS as u16).clamp(1, MAX_BAR_W);
        Self { bar_w, gap, show_sides }
    }

    /// Total columns used by the bars (including gaps).
    pub fn bars_width(&self) -> u16 {
        self.bar_w * SPECTRUM_BANDS as u16 + self.gap * (SPECTRUM_BANDS as u16 - 1)
    }

    fn cells(&self) -> f64 {
        (self.bar_w * SPECTRUM_BANDS as u16) as f64
    }
}

pub fn lerp(c1: Color, c2: Color, t: f64) -> Color {
    match (c1, c2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8;
            Color::Rgb(mix(r1, r2), mix(g1, g2), mix(b1, b2))
        }
        _ => if t > 0.5 { c2 } else { c1 },
    }
}

/// One spectrum row (`r`: 0 = bottom, 2 = top). Played cells use `color`,
/// pending cells a faint tint of it; `progress` is 0.0..=1.0.
pub fn spectrum_row(spec: &SpectrumFrame, r: u8, color: Color, layout: BarLayout, progress: f64) -> Vec<Span<'static>> {
    let pending = lerp(TRACK_BG, color, PENDING_MIX);
    let progress_cell = progress.clamp(0.0, 1.0) * layout.cells();
    let mut v = Vec::with_capacity(layout.bars_width() as usize);
    for (b, &val) in spec.iter().enumerate() {
        let idx = if val == 0 && r == 0 { 1 } else { val.saturating_sub(r * 8).min(8) as usize };
        let glyph = BAR_GLYPHS[idx];
        for c in 0..layout.bar_w {
            let cell = (b as u16 * layout.bar_w + c) as f64;
            let col = if cell + 1.0 <= progress_cell {
                color
            } else if cell <= progress_cell {
                lerp(pending, color, progress_cell - cell)
            } else {
                pending
            };
            v.push(Span::styled(glyph, Style::new().fg(col)));
        }
        if layout.gap > 0 && b + 1 < SPECTRUM_BANDS {
            v.push(Span::raw(" "));
        }
    }
    v
}
#[cfg(test)]
#[path = "player_spectrum_tests.rs"]
mod tests;
