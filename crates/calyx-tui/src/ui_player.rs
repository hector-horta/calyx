//! Player panel in two boxes: the song (cover, title, up next; player_track.rs)
//! and the meters (spectrum, time, notices and the audio status).

use crate::app::TuiApp;
use crate::hints::{LABEL_STYLE, hint_bar_fit};
use crate::player_spectrum::{BarLayout, spectrum_row};
use crate::player_status::{progress, status_lines};
use crate::ui::truncate;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

/// Bar colours from the top row down.
const ROW_COLORS: [Color; 3] = [
    Color::Rgb(255, 215, 0),
    Color::Rgb(0, 240, 180),
    Color::Rgb(0, 200, 100),
];
const INFO: Style = Style::new().fg(Color::Rgb(120, 160, 160));

pub fn render_player_panel(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let song_w = (area.width * 45 / 100).clamp(30, 72);
    let [song, meters] =
        Layout::horizontal([Constraint::Length(song_w), Constraint::Min(30)]).areas(area);
    crate::player_track::render_track_box(frame, song, app);
    render_meters_box(frame, meters, app);
}

fn render_meters_box(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let l = app.language;
    let keys = [
        ("Enter", l.btn_play(), LABEL_STYLE),
        (l.key_space(), l.btn_pause(), LABEL_STYLE),
        ("< / >", l.btn_prev_next(), LABEL_STYLE),
        ("- / +", l.btn_volume(), LABEL_STYLE),
    ];
    let bar = hint_bar_fit(&keys, area.width.saturating_sub(2) as usize);
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::right(crate::player_track::RIGHT_MARGIN))
        .title_bottom(bar);
    let inner_w = block.inner(area).width;
    let layout = BarLayout::for_width(inner_w);
    let (_, _, ratio) = progress(app);
    let spec = app.current_spectrum();

    // The spectrum alone, centred: time, controls and status share its axis.
    let mut lines: Vec<Line<'static>> = ROW_COLORS
        .iter()
        .enumerate()
        .map(|(i, color)| Line::from(spectrum_row(&spec, (2 - i) as u8, *color, layout, ratio)))
        .collect();
    lines.extend(status_lines(app, inner_w as usize));
    // No fixed blank rows: when there is no notice its slot is the only gap.
    lines.push(notice_line(app, inner_w as usize));
    lines.push(Line::from(Span::styled(app.audio_hint.clone(), INFO)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [body] = Layout::vertical([Constraint::Length(lines.len() as u16)])
        .flex(Flex::Center)
        .areas(inner);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), body);
}

/// The player zone's live action/error/result in reverse video; otherwise
/// its progress message (with the spinner while the audio loads), plain.
fn notice_line(app: &TuiApp, width: usize) -> Line<'static> {
    let slot = &app.zones.player;
    if let Some(notice) = slot.notice.as_ref().filter(|n| n.is_live()) {
        return notice.line(width);
    }
    let progress = app
        .activity
        .decorate_jobs(&[crate::activity::Job::Audio], &slot.status);
    let style = INFO.add_modifier(Modifier::ITALIC);
    Line::from(Span::styled(
        truncate(&progress, width.saturating_sub(1)),
        style,
    ))
}
