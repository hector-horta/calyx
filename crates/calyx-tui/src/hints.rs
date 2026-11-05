//! Single visual format for every keyboard hint: bold cyan key + dim label.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub const KEY_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
pub const LABEL_STYLE: Style = Style::new().fg(Color::DarkGray);
const SEP: &str = " ── ";

/// `key label` with the standard styles; `label_style` lets callers tint the label.
pub fn hint(key: &'static str, label: &'static str, label_style: Style) -> [Span<'static>; 3] {
    [
        Span::styled(key, KEY_STYLE),
        Span::raw(" "),
        Span::styled(label, label_style)
    ]
}

/// A border title like `─a Encolar ── n A continuación─`.
pub fn hint_bar(items: &[(&'static str, &'static str, Style)]) -> Line<'static> {
    let mut spans = Vec::with_capacity(1 + items.len() * 3 + items.len().saturating_sub(1));
    spans.push(Span::raw("─"));
    for (i, (key, label, style)) in items.iter().enumerate() {
        if i > 0 { spans.push(Span::raw(SEP)); }
        spans.extend(hint(key, label, *style));
    }
    spans.push(Span::raw("─"));
    Line::from(spans)
}

/// A navigation tab: same `key label` format; the active tab is shown inverted.
pub fn tab(key: &'static str, label: &'static str, active: bool) -> [Span<'static>; 3] {
    if active {
        let s = Style::new().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD);
        [
            Span::styled(key, s),
            Span::styled(" ", s),
            Span::styled(label, s)
        ]
    } else {
        hint(key, label, Style::new().fg(Color::Cyan))
    }
}

/// Joins tabs with the standard separator, wrapped in border dashes.
pub fn tab_bar(tabs: Vec<[Span<'static>; 3]>) -> Line<'static> {
    let mut spans = Vec::with_capacity(1 + tabs.len() * 3 + tabs.len().saturating_sub(1));
    spans.push(Span::raw("─"));
    for (i, t) in tabs.into_iter().enumerate() {
        if i > 0 { spans.push(Span::raw(SEP)); }
        spans.extend(t);
    }
    spans.push(Span::raw("─"));
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(line: &Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn hint_bar_joins_items() {
        let line = hint_bar(&[("a", "Encolar", LABEL_STYLE), ("n", "A continuación", LABEL_STYLE)]);
        assert_eq!(text(&line), "─a Encolar ── n A continuación─");
    }

    #[test]
    fn tab_bar_joins_tabs() {
        let line = tab_bar(vec![tab("1", "Inicio", true), tab("2", "Novedades", false)]);
        assert_eq!(text(&line), "─1 Inicio ── 2 Novedades─");
    }
}