
use super::*;

fn text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn hint_bar_joins_items() {
    let line = hint_bar(&[
        ("a", "Encolar", LABEL_STYLE),
        ("n", "A continuación", LABEL_STYLE),
    ]);
    assert_eq!(text(&line), "─[a] Encolar ── [n] A continuación─");
}

#[test]
fn fitted_bar_drops_trailing_hints_until_it_fits() {
    let items = [
        ("a", "Encolar", LABEL_STYLE),
        ("n", "A continuación", LABEL_STYLE),
        ("c", "Limpiar", LABEL_STYLE),
    ];
    let full = hint_bar(&items);
    assert_eq!(text(&hint_bar_fit(&items, 200)), text(&full));
    let two = text(&hint_bar_fit(&items, full.width() - 1));
    assert_eq!(two, "─[a] Encolar ── [n] A continuación─");
    // Never drops the first hint, even if it does not fit.
    assert_eq!(text(&hint_bar_fit(&items, 3)), "─[a] Encolar─");
}

#[test]
fn tab_bar_joins_tabs() {
    let line = tab_bar(vec![tab("1", "Inicio", true), tab("2", "Novedades", false)]);
    assert_eq!(text(&line), "─[1] Inicio ── [2] Novedades─");
}
