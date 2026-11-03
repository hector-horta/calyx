use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::TuiApp;

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() || max_width < 2 { return vec![text.to_string()] }
    let mut lines = Vec::new();
    let mut current_line = String::with_capacity(max_width);
    let mut words = text.split_whitespace();

    for word in words.by_ref() {
        if current_line.is_empty() || current_line.len() + 1 + word.len() <= max_width {
            if !current_line.is_empty() { current_line.push(' '); }
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() { lines.push(current_line) }
    lines
}

pub fn render_lyrics(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let empty_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            app.language.no_lyrics(),
            Style::default().fg(Color::DarkGray)
        )),
    ];

    if app.current_lyrics.is_empty() {
        frame.render_widget(
            Paragraph::new(empty_lines).block(Block::default()
                .borders(Borders::LEFT)
                .title_top(app.language.title_lyrics())
            ),
            area
        );
        return;
    }

    let (elap, _) = app.playback_progress();
    let cur_ms = elap.as_millis() as u64;
    let max_w = (area.width as usize).saturating_sub(6).max(10);
    let active_idx = app.current_lyrics.iter()
        .rposition(|l| l.time_ms <= cur_ms)
        .unwrap_or(0);

    let mut visual_lines: Vec<Line<'static>> = Vec::new();
    let mut active_visual_idx = 0;
    for (idx, line) in app.current_lyrics.iter().enumerate() {
        let is_active = idx == active_idx;
        let chunks = wrap_text(&line.text, max_w);

        for (c_idx, chunk) in chunks.into_iter().enumerate() {
            if is_active {
                if c_idx == 0 {
                    active_visual_idx = visual_lines.len();
                    visual_lines.push(Line::from(vec![
                        Span::styled(" ♪ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(chunk, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    visual_lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled(chunk, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]));
                }
            } else {
                visual_lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(chunk, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    }

    let height = area.height as usize;
    let start = active_visual_idx.saturating_sub(height / 2);
    let visible: Vec<Line<'static>> = visual_lines.into_iter().skip(start).take(height).collect();
    frame.render_widget(
        Paragraph::new(visible)
            .block(Block::default()
                .borders(Borders::LEFT)
                .title_top(app.language.title_lyrics())),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_long_line() {
        let text = "And it's stickin' out, yeah, like a sore thumb (blowin' up)";
        let wrapped = wrap_text(text, 28);
        assert!(wrapped.len() >= 2);
        assert_eq!(wrapped[0], "And it's stickin' out, yeah,");
    }
}