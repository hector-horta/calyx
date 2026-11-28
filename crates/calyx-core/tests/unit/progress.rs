use super::*;

#[test]
fn counts_pages_and_reports_total_once_known() {
    let p = LoadProgress::default();
    assert_eq!(p.snapshot(), (0, None));
    p.add(100);
    p.add(100);
    assert_eq!(p.snapshot(), (200, None));
    p.set_total(1000);
    assert_eq!(p.snapshot(), (200, Some(1000)));
}
