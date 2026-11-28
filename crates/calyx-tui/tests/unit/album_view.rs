
use super::*;

#[test]
fn albums_from_non_list_views_open_in_home() {
    assert_eq!(album_view_for(ActiveView::Library), ActiveView::Library);
    assert_eq!(album_view_for(ActiveView::New), ActiveView::New);
    assert_eq!(album_view_for(ActiveView::Queue), ActiveView::Home);
    assert_eq!(album_view_for(ActiveView::Playlists), ActiveView::Home);
}
