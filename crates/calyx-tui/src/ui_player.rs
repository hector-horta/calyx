//! Player panel in two boxes: the song (cover, title, up next; player_track.rs)
//! and the meters (spectrum, time, notices and the audio status).

use crate::app::TuiApp;
use crate::hints::{LABEL_STYLE, hint_bar_fit};
use crate::player_spectrum::{BarLayout, spectrum_row};
use crate::player_status::{Density, progress, status_lines};
use crate::ui::truncate;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
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
const OFF: Style = Style::new().fg(Color::DarkGray);
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

    let mut lines = Vec::new();
    for (i, color) in ROW_COLORS.iter().enumerate() {
        let r = (2 - i) as u8;
        let mut row = spectrum_row(&spec, r, *color, layout, ratio);
        if layout.show_sides {
            row.push(Span::raw("  "));
            row.extend(volume_cell(app, r));
        }
        lines.push(Line::from(row));
    }
    lines.extend(status_lines(app, Density::for_width(inner_w)));
    lines.push(Line::from(""));
    lines.push(notice_line(app, inner_w as usize));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(app.audio_hint.clone(), INFO)));
    let body = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(body, area);
}

/// A live action/error/result in reverse video; otherwise the routine
/// progress message (with the activity spinner and counter), plain.
fn notice_line(app: &TuiApp, width: usize) -> Line<'static> {
    if let Some(notice) = app.notice.as_ref().filter(|n| n.is_live()) {
        return notice.line(width);
    }
    let progress = app.activity.decorate(&app.status_message);
    let style = INFO.add_modifier(Modifier::ITALIC);
    Line::from(Span::styled(
        truncate(&progress, width.saturating_sub(1)),
        style,
    ))
}

/// Volume meter column (6 columns): label, percentage, then the step bar.
fn volume_cell(app: &TuiApp, r: u8) -> Vec<Span<'static>> {
    let vol = (app.volume * 100.0).round() as u8;
    match r {
        2 => vec![Span::styled("   VOL", OFF)],
        1 => vec![Span::styled(
            format!("  {:>4}", format!("{vol}%")),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        )],
        _ => {
            let lit = ((vol as f64 / 100.0) * 4.0).round() as usize;
            let steps = ["▂", "▄", "▆", "█"].into_iter().enumerate().map(|(i, s)| {
                Span::styled(
                    s,
                    if i < lit {
                        Style::new().fg(Color::Cyan)
                    } else {
                        OFF
                    },
                )
            });
            std::iter::once(Span::raw("  ")).chain(steps).collect()
        }
    }
}
