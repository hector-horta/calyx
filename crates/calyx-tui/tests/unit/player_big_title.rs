use super::scroll_offset;

#[test]
fn a_title_that_fits_never_moves() {
    for ms in [0, 2500, 10_000, 200_000] {
        assert_eq!(scroll_offset(ms, 0), 0);
    }
}

#[test]
fn a_long_title_scrolls_once_then_rests_on_its_start() {
    assert_eq!(scroll_offset(1000, 20), 0); // still at first
    assert_eq!(scroll_offset(2000 + 900, 20), 10); // half way
    assert_eq!(scroll_offset(2000 + 1800 + 500, 20), 20); // end hold
    assert_eq!(scroll_offset(60_000, 20), 0); // back to the start
}
