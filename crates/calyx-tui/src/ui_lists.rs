//! Track, queue and playlist lists of the central panel, with column headers.

use crate::app::TuiApp;
use crate::spinner::cache_cell;
use crate::ui::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

const HEADER_STYLE: Style = Style::new()
    .fg(Color::Rgb(120, 160, 160))
    .add_modifier(Modifier::BOLD);
const EMPTY_STYLE: Style = Style::new().fg(Color::DarkGray);
const SPIN_STYLE: Style = Style::new().fg(Color::Green);
/// Blank gutter matching the list highlight symbol ("▶ ") so headers line up.
const MARK: &str = "  ";
/// Blank header cell over the cache-spinner column (spinner + space).
const SPIN_COL: &str = "  ";

/// List row with the cache-spinner column between `lead` and `rest`.
fn row(lead: String, cell: &'static str, rest: String) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::raw(lead),
        Span::styled(cell, SPIN_STYLE),
        Span::raw(" "),
        Span::raw(rest),
    ]))
}

fn split_header(area: Rect) -> (Rect, Rect) {
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    (parts[0], parts[1])
}

fn header(frame: &mut Frame<'_>, area: Rect, text: String) {
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, HEADER_STYLE))),
        area,
    );
}

fn highlighted(items: Vec<ListItem<'static>>, bg: Color) -> List<'static> {
    List::new(items).highlight_symbol("▶ ").highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
    )
}

pub fn render_playlists(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let items = if app.playlists.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            format!("{MARK}{}", app.language.msg_no_playlists()),
            EMPTY_STYLE,
        )))]
    } else {
        app.playlists
            .iter()
            .map(|p| {
                ListItem::new(Line::from(format!(
                    "📁 {:<36} │ {}",
                    truncate(&p.name, 34),
                    truncate(&p.description, 36)
                )))
            })
            .collect()
    };
    let mut state = app.playlist_state.clone();
    frame.render_stateful_widget(highlighted(items, Color::Cyan), area, &mut state);
}

pub fn render_queue(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let l = app.language;
    let upcoming = app.queue.upcoming();
    let avail = (area.width as usize).saturating_sub(18);
    let (wt, wa) = if avail < 30 {
        (28, 16)
    } else {
        let wt = (avail * 60) / 100;
        (wt, avail.saturating_sub(wt))
    };
    let wt = wt.saturating_sub(SPIN_COL.len());
    let (head, body) = split_header(area);
    header(
        frame,
        head,
        format!(
            "{MARK}{:>2}  {SPIN_COL}{:<wt$} │ {:<wa$} │ {}",
            "#",
            truncate(l.col_title(), wt.saturating_sub(1)),
            truncate(l.col_artist(), wa.saturating_sub(1)),
            l.col_duration()
        ),
    );
    let items = if upcoming.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            format!("{MARK}{}", l.msg_queue_empty()),
            EMPTY_STYLE,
        )))]
    } else {
        upcoming
            .iter()
            .enumerate()
            .map(|(i, t)| {
                row(
                    format!("{:>2}. ", i + 1),
                    cache_cell(&app.caching, &t.id),
                    format!(
                        "{:<wt$} │ {:<wa$} │ {}",
                        truncate(&t.title, wt.saturating_sub(1)),
                        truncate(&t.artist, wa.saturating_sub(1)),
                        t.formatted_duration()
                    ),
                )
            })
            .collect()
    };
    let mut state = app.queue_state.clone();
    frame.render_stateful_widget(highlighted(items, Color::Yellow), body, &mut state);
}

pub fn render_tracks(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let l = app.language;
    let avail = (area.width as usize).saturating_sub(18);
    let (wt, wa, wal) = if avail < 40 {
        (26, 16, 14)
    } else {
        let wt = (avail * 46) / 100;
        let wa = (avail * 24) / 100;
        (wt, wa, avail.saturating_sub(wt + wa))
    };
    let wt = wt.saturating_sub(SPIN_COL.len());
    let (head, body) = split_header(area);
    header(
        frame,
        head,
        format!(
            "{MARK}{SPIN_COL}{:<wt$} │ {:<wa$} │ {:<wal$} │ {}",
            truncate(l.col_title(), wt.saturating_sub(1)),
            truncate(l.col_artist(), wa.saturating_sub(1)),
            truncate(l.col_album(), wal.saturating_sub(1)),
            l.col_duration()
        ),
    );
    let items = app
        .tracks
        .iter()
        .map(|t| {
            row(
                String::new(),
                cache_cell(&app.caching, &t.id),
                format!(
                    "{:<wt$} │ {:<wa$} │ {:<wal$} │ {}",
                    truncate(&t.title, wt.saturating_sub(1)),
                    truncate(&t.artist, wa.saturating_sub(1)),
                    truncate(&t.album, wal.saturating_sub(1)),
                    t.formatted_duration()
                ),
            )
        })
        .collect();
    let mut state = app.list_state.clone();
    frame.render_stateful_widget(highlighted(items, Color::Cyan), body, &mut state);
}
