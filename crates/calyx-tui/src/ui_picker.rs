//! Popup of the 'm' picker, drawn over the central panel.

use crate::app::TuiApp;
use crate::hints::{LABEL_STYLE, hint_bar};
use crate::picker_texts::{PickerTexts, picker_texts};
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

/// Displays a text field for entering a new playlist name.
fn naming_popup(frame: &mut Frame<'_>, texts: &PickerTexts, name: &str) {
    let area = centered(frame.area(), 64, 3);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan))
        .title_top(texts.new_title.clone())
        .title_bottom(hint_bar(&[
            ("Enter", texts.create, LABEL_STYLE),
            ("Esc", texts.back, LABEL_STYLE),
        ]));
    let line = Line::from(vec![
        Span::styled(texts.name_label, Style::new().fg(Color::DarkGray)),
        Span::raw(name.to_string()),
        Span::styled("█", Style::new().fg(Color::Yellow)),
    ]);
    frame.render_widget(Clear, area);
    frame.render_widget(ratatui::widgets::Paragraph::new(line).block(block), area);
}

pub fn render_picker(frame: &mut Frame<'_>, app: &TuiApp) {
    let Some(picker) = &app.picker else {
        return;
    };
    let song = truncate(&picker.track.title, SONG_TITLE_CHARS);
    let texts = picker_texts(app.language, &song);
    if let Some(name) = &picker.naming {
        naming_popup(frame, &texts, name);
        return;
    }
    let playlists = app.editable_playlists();
    let mut items = vec![ListItem::new(Line::from(texts.new_row))];
    if playlists.is_empty() {
        let text = if app.playlists.is_empty() {
            texts.loading
        } else {
            texts.none
        };
        items.push(ListItem::new(Span::styled(
            text,
            Style::new().fg(Color::DarkGray),
        )));
    } else {
        items.extend(
            playlists
                .iter()
                .map(|p| ListItem::new(Line::from(format!("📁 {}", p.name)))),
        );
    }

    let rows = (items.len() as u16).clamp(1, MAX_ROWS);
    let area = centered(frame.area(), 64, rows + 2);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan))
        .title_top(texts.title)
        .title_bottom(hint_bar(&[
            ("Enter", texts.add, LABEL_STYLE),
            ("n", texts.new_key, LABEL_STYLE),
            ("Esc", texts.cancel, LABEL_STYLE),
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
