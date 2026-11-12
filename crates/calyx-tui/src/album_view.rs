//! 'o' toggles between a track list and the album of its selected song: the
//! list on screen is kept while the album is shown, so going back is instant
//! (no reload of a 1000-song library) and keeps the selection.

use crate::activity::Job;
use crate::app::{ActiveView, TuiApp};
use crate::bg_events::BgEvent;
use crate::messages::Msg;
use calyx_core::{StreamingBackend, Track};

/// The list that was on screen when an album was opened.
pub struct SavedList {
    pub view: ActiveView,
    pub tracks: Vec<Track>,
    pub selected: usize,
}

/// View an album opens in: track-list views keep their tab, the rest use Home.
fn album_view_for(current: ActiveView) -> ActiveView {
    match current {
        ActiveView::Queue | ActiveView::Help | ActiveView::Playlists => ActiveView::Home,
        view => view,
    }
}

impl TuiApp {
    /// True while an album opened with 'o' is the visible list.
    pub fn showing_album(&self) -> bool {
        self.album_back.is_some()
            && !matches!(
                self.active_view,
                ActiveView::Queue | ActiveView::Help | ActiveView::Playlists
            )
    }

    /// 'o': open the album of the selected song, or go back from an album.
    pub fn toggle_album(&mut self) {
        if self.showing_album() {
            self.back_from_album();
            return;
        }
        let Some(track) = self.selected_track() else {
            return;
        };
        self.say(Msg::LoadingAlbum(&track.title));
        self.activity.start(Job::List);
        let backend = self.backend.clone();
        self.spawn_list_event(async move {
            BgEvent::AlbumOpened(
                backend
                    .fetch_album_of(&track)
                    .await
                    .map_err(|e| e.to_string()),
            )
        });
    }

    fn back_from_album(&mut self) {
        let Some(saved) = self.album_back.take() else {
            return;
        };
        self.tracks = saved.tracks;
        self.selected_index = saved.selected.min(self.tracks.len().saturating_sub(1));
        self.list_state
            .select((!self.tracks.is_empty()).then_some(self.selected_index));
        self.active_view = saved.view;
        self.say(Msg::BackToList(crate::labels::view_name(
            self.language,
            saved.view,
        )));
    }

    /// Shows a freshly loaded album, remembering the list it replaces. If an
    /// album was already open (another one opened from the queue), the list
    /// saved before it is kept, so 'o' still leads back to the original list.
    pub(crate) fn show_album(&mut self, name: &str, tracks: Vec<Track>) {
        let album_view = album_view_for(self.active_view);
        let saved = self.album_back.take().unwrap_or_else(|| SavedList {
            view: self.active_view,
            tracks: std::mem::take(&mut self.tracks),
            selected: self.selected_index,
        });
        self.say(Msg::AlbumOpened {
            name,
            count: tracks.len(),
            back: crate::labels::view_name(self.language, saved.view),
        });
        self.tracks = tracks;
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = album_view;
        self.album_back = Some(saved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn albums_from_non_list_views_open_in_home() {
        assert_eq!(album_view_for(ActiveView::Library), ActiveView::Library);
        assert_eq!(album_view_for(ActiveView::New), ActiveView::New);
        assert_eq!(album_view_for(ActiveView::Queue), ActiveView::Home);
        assert_eq!(album_view_for(ActiveView::Playlists), ActiveView::Home);
    }
}
