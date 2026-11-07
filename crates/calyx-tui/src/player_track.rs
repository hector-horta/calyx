//! Left box of the player: cover art, the song (artist, title, album) with the
//! big play/pause icon, and what plays next at the bottom.

use crate::app::TuiApp;
use crate::artwork::{ART_WIDTH, TerminalCoverArt};
use crate::ui::truncate;
use calyx_core::models::PlaybackStatus;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

const OFF: Style = Style::new().fg(Color::DarkGray);
/// Big play/pause icon: 6 columns x 3 rows.
const ICON_W: u16 = 6;
/// Below this inner width the cover is hidden so the text stays readable.
const MIN_TEXT_W: u16 = 16;
/// Rows of the play/pause icon.
const ICON_H: u16 = 3;
/// Columns between the icon and the text.
const ICON_GAP: u16 = 2;
/// Blank column between the content and the right border (both player boxes).
pub const RIGHT_MARGIN: u16 = 1;

pub fn render_track_box(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::right(RIGHT_MARGIN));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    // cover | icon | text: the icon sits next to the cover like a play button,
    // so the text starts right after it with no gap to fill.
    let art_w = ART_WIDTH as u16 + 1;
    let icon_w = ICON_W + ICON_GAP;
    let show_art = inner.width >= art_w + icon_w + MIN_TEXT_W;
    let [art, icon, text] = Layout::horizontal([
        Constraint::Length(if show_art { art_w } else { 0 }),
        Constraint::Length(icon_w),
        Constraint::Min(0),
    ])
    .areas(inner);
    if show_art {
        let cover = match &app.current_artwork {
            Some(a) => a.to_lines(),
            None => TerminalCoverArt::from_jpeg_or_seed(None, "Calyx").to_lines(),
        };
        frame.render_widget(Paragraph::new(cover), art);
    }
    let [icon] = Layout::vertical([Constraint::Length(ICON_H)])
        .flex(Flex::Center)
        .areas(icon);
    frame.render_widget(Paragraph::new(icon_lines(app)), icon);
    // The song (and what plays next) as one block, centred like the icon.
    let lines = track_lines(app, text.width as usize);
    let [text] = Layout::vertical([Constraint::Length(lines.len() as u16)])
        .flex(Flex::Center)
        .areas(text);
    frame.render_widget(Paragraph::new(lines), text);
}

/// The big play/pause icon, 3 rows of 6 columns.
fn icon_lines(app: &TuiApp) -> Vec<Line<'static>> {
    let icon = match app.playback_status() {
        PlaybackStatus::Playing => &app.play_icon.rows,
        PlaybackStatus::Paused | PlaybackStatus::Stopped => &app.pause_icon.rows,
        PlaybackStatus::Buffering => {
            let cyan = Style::new().fg(Color::Cyan);
            return vec![Line::from(""), Line::from(Span::styled("  ↻", cyan))];
        }
    };
    icon.iter().map(|row| Line::from(row.clone())).collect()
}

/// Returns the lines for the track box showing current song and next track.
fn track_lines(app: &TuiApp, width: usize) -> Vec<Line<'static>> {
    let l = app.language;
    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(t) = &app.current_track {
        lines.push(Line::from(Span::styled(
            truncate(&t.artist, width),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            truncate(&t.title, width),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(truncate(&t.album, width), OFF)));
    } else {
        lines.push(Line::from(Span::styled(l.no_active_playback(), OFF)));
    }

    if let Some(next) = app.queue.peek_next() {
        let label = format!(
            "{} ({} {}):",
            l.lbl_up_next(),
            app.queue.len(),
            l.lbl_in_queue()
        );
        let song = format!("{} - {}", next.title, next.artist);
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(truncate(&label, width), OFF)));
        lines.push(Line::from(Span::styled(
            truncate(&song, width),
            Style::new().fg(Color::Rgb(120, 160, 160)),
        )));
    }

    lines
}
