use crate::app::{ActiveView, TuiApp};

impl TuiApp {
    /// Opens the home view: recently searched tracks from the local store.
    pub fn open_home(&mut self) {
        self.album_back = None;
        self.tracks = self.store.search_tracks("", 50).unwrap_or_default();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = ActiveView::Home;
        self.say(crate::messages::Msg::RecentOpened);
    }

    pub fn next_track(&mut self) {
        match self.active_view {
            ActiveView::Help => {}
            ActiveView::Playlists => {
                if !self.playlists.is_empty() {
                    let next =
                        (self.playlist_state.selected().unwrap_or(0) + 1) % self.playlists.len();
                    self.playlist_state.select(Some(next));
                }
            }
            ActiveView::Queue => {
                if !self.queue.is_empty() {
                    let next = (self.queue_state.selected().unwrap_or(0) + 1) % self.queue.len();
                    self.queue_state.select(Some(next));
                }
            }
            _ => {
                if !self.tracks.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.tracks.len();
                    self.list_state.select(Some(self.selected_index));
                }
            }
        }
    }

    pub fn previous_track(&mut self) {
        match self.active_view {
            ActiveView::Help => {}
            ActiveView::Playlists => {
                if !self.playlists.is_empty() {
                    let cur = self.playlist_state.selected().unwrap_or(0);
                    self.playlist_state.select(Some(if cur == 0 {
                        self.playlists.len() - 1
                    } else {
                        cur - 1
                    }));
                }
            }
            ActiveView::Queue => {
                if !self.queue.is_empty() {
                    let cur = self.queue_state.selected().unwrap_or(0);
                    self.queue_state.select(Some(if cur == 0 {
                        self.queue.len() - 1
                    } else {
                        cur - 1
                    }));
                }
            }
            _ => {
                if !self.tracks.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.tracks.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                    self.list_state.select(Some(self.selected_index));
                }
            }
        }
    }

    pub fn toggle_view(&mut self) {
        self.active_view = if self.active_view == ActiveView::Queue {
            ActiveView::Home
        } else {
            ActiveView::Queue
        };
    }
}
