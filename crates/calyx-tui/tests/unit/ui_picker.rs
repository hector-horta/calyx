
use super::*;

#[test]
fn popup_is_centered_and_clamped() {
    let screen = Rect::new(0, 0, 100, 40);
    assert_eq!(centered(screen, 64, 10), Rect::new(18, 15, 64, 10));
    let small = Rect::new(0, 0, 50, 6);
    assert_eq!(centered(small, 64, 10), Rect::new(0, 0, 50, 6));
}
