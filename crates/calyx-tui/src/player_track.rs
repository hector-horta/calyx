//! Left box of the player: cover art, then the song as one centred block (the
//! playback state, artist, title in big letters, album) and what plays next.

use crate::app::TuiApp;
use crate::artwork::{ART_WIDTH, TerminalCoverArt};
use crate::player_big_title::{BIG_H, big_title};
use crate::ui::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

const OFF: Style = Style::new().fg(Color::DarkGray);
/// Below this inner width the cover is hidden so the text stays readable.
const MIN_TEXT_W: u16 = 16;
/// Columns between the cover and the text.
const ART_GAP: u16 = 2;
/// Blank column between the content and the right border (both player boxes).
pub const RIGHT_MARGIN: u16 = 1;

pub fn render_track_box(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::right(RIGHT_MARGIN));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let art_w = ART_WIDTH as u16 + ART_GAP;
    let show_art = inner.width >= art_w + MIN_TEXT_W;
    let [art, text] = Layout::horizontal([
        Constraint::Length(if show_art { art_w } else { 0 }),
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
    // The song (and what plays next) as one block, centred vertically.
    let mut lines = track_lines(app, text.width as usize);
    let big = big_title(app, text.width);
    // The big title takes the place of the plain title line.
    let big_h = if big.is_some() { BIG_H - 1 } else { 0 };
    let [text] = Layout::vertical([Constraint::Length(lines.len() as u16 + big_h)])
        .flex(Flex::Center)
        .areas(text);
    let Some(big) = big else {
        frame.render_widget(Paragraph::new(lines), text);
        return;
    };
    // State and artist, then the title in big letters instead of its line.
    let rest = lines.split_off(3);
    lines.truncate(2);
    let [head, title, below] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(BIG_H),
        Constraint::Min(0),
    ])
    .areas(text);
    frame.render_widget(Paragraph::new(lines), head);
    big.render(frame, title);
    frame.render_widget(Paragraph::new(rest), below);
}

/// The state, artist, title and album, then what plays next.
fn track_lines(app: &TuiApp, width: usize) -> Vec<Line<'static>> {
    let l = app.language;
    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(t) = &app.current_track {
        lines.push(crate::player_state::state_line(app));
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
