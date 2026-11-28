
use super::*;

#[test]
fn labels_follow_the_language() {
    assert_eq!(album(Language::Es), "Álbum");
    assert_eq!(album_to_library(Language::En), "Save album");
    assert_eq!(back_to(Language::Es, ActiveView::Library), "← Biblioteca");
    assert_eq!(view_name(Language::Es, ActiveView::Library), "Biblioteca");
}
