use super::step;

#[test]
fn selection_wraps_both_ways() {
    assert_eq!(step(0, 3, true), 1);
    assert_eq!(step(2, 3, true), 0);
    assert_eq!(step(0, 3, false), 2);
    assert_eq!(step(1, 3, false), 0);
    assert_eq!(step(0, 0, true), 0);
}
