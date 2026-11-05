//! Unit tests for `player_spectrum` (included via `#[path]`).

use super::*;
use ratatui::style::Color;

const GREEN: Color = Color::Rgb(0, 200, 100);
const SIDE_COLS: u16 = 4;
const SPECTRUM_BANDS: usize = 12;

fn used_cols(l: BarLayout) -> u16 {
    if l.show_sides { SIDE_COLS + l.bars_width() } else { l.bars_width() }
}

#[test]
fn wide_panel_widens_bars() {
    let l = BarLayout::for_width(120);
    assert!(l.show_sides);
    assert_eq!(l.bar_w, 7);
}

#[test]
fn medium_panel_keeps_icon_and_volume() {
    let l = BarLayout::for_width(70);
    assert!(l.show_sides);
    assert_eq!(l.bar_w, 3);
}

#[test]
fn narrow_panel_drops_sides_and_gaps() {
    let l = BarLayout::for_width(40);
    assert!(!l.show_sides);
    assert_eq!((l.bar_w, l.gap), (3, 0));
}

#[test]
fn never_wider_than_panel() {
    for w in 12..=200u16 {
        assert!(used_cols(BarLayout::for_width(w)) <= w, "overflow at width {w}");
    }
}

#[test]
fn row_width_matches_layout() {
    let l = BarLayout::for_width(100);
    let row = spectrum_row(&[12; SPECTRUM_BANDS], 0, GREEN, l, 0.5);
    let width: usize = row.iter().map(|s| s.width()).sum();
    assert_eq!(width, l.bars_width() as usize);
}

#[test]
fn pending_bars_are_tinted_not_full_color() {
    let l = BarLayout::for_width(100);
    let row = spectrum_row(&[12; SPECTRUM_BANDS], 0, GREEN, l, 0.0);
    assert!(row.iter().filter(|s| s.content != " ").all(|s| s.style.fg != Some(GREEN)));
}

#[test]
fn played_bars_use_full_color() {
    let l = BarLayout::for_width(100);
    let row = spectrum_row(&[12; SPECTRUM_BANDS], 0, GREEN, l, 1.0);
    assert!(row.iter().filter(|s| s.content != " ").all(|s| s.style.fg == Some(GREEN)));
}