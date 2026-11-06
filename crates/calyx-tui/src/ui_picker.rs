//! Popup of the 'm' picker, drawn over the central panel.

use crate::app::TuiApp;
use crate::hints::{LABEL_STYLE, hint_bar};
use crate::ui::truncate;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

const MAX_ROWS: u16 = 12;
const SONG_TITLE_CHARS: usize = 40;

/// A box of `width` x `height` centered in `area` (clamped to it).
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height.min(area.height))])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width.min(area.width))])
        .flex(Flex::Center)
        .areas(row);
    cell
}

pub fn render_picker(frame: &mut Frame<'_>, app: &TuiApp) {
    let Some(picker) = &app.picker else {
        return;
    };
    let song = truncate(&picker.track.title, SONG_TITLE_CHARS);
    let (title, add, cancel, loading, none) = crate::labels::picker(app.language, &song);
    let playlists = app.editable_playlists();
    let items: Vec<ListItem<'static>> = if playlists.is_empty() {
        let text = if app.playlists.is_empty() {
            loading
        } else {
            none
        };
        vec![ListItem::new(Span::styled(
            text,
            Style::new().fg(Color::DarkGray),
        ))]
    } else {
        playlists
            .iter()
            .map(|p| ListItem::new(Line::from(format!("📁 {}", p.name))))
            .collect()
    };

    let rows = (items.len() as u16).clamp(1, MAX_ROWS);
    let area = centered(frame.area(), 64, rows + 2);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan))
        .title_top(title)
        .title_bottom(hint_bar(&[
            ("Enter", add, LABEL_STYLE),
            ("Esc", cancel, LABEL_STYLE),
        ]));
    let list = List::new(items)
        .block(block)
        .highlight_symbol("▶ ")
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = picker.state.clone();
    if playlists.is_empty() {
        state.select(None);
    }
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_is_centered_and_clamped() {
        let screen = Rect::new(0, 0, 100, 40);
        assert_eq!(centered(screen, 64, 10), Rect::new(18, 15, 64, 10));
        let small = Rect::new(0, 0, 50, 6);
        assert_eq!(centered(small, 64, 10), Rect::new(0, 0, 50, 6));
    }
}
