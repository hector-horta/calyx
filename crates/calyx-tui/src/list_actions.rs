//! Actions on the selected row of the visible list: enqueue, remove, clear.

use crate::app::{ActiveView, TuiApp};
use crate::messages::Msg;

impl TuiApp {
    pub fn enqueue_selected_back(&mut self) {
        if let Some(track) = self.selected_track() {
            self.say(Msg::Enqueued {
                artist: &track.artist,
                title: &track.title,
            });
            self.queue.enqueue_back(track);
        }
    }

    pub fn enqueue_selected_next(&mut self) {
        if let Some(track) = self.selected_track() {
            self.say(Msg::EnqueuedNext {
                artist: &track.artist,
                title: &track.title,
            });
            self.queue.enqueue_next(track);
        }
    }

    /// 'd': remove the selected row from the queue, from Recent (Home) or, inside
    /// an opened playlist, from that playlist on Apple Music (asks first).
    pub fn remove_selected_from_queue(&mut self) {
        if self.active_view == ActiveView::Library && self.open_playlist.is_some() {
            self.remove_selected_from_playlist();
        } else if self.active_view == ActiveView::Queue {
            if let Some(idx) = self.queue_state.selected()
                && let Some(removed) = self.queue.remove(idx)
            {
                self.say(Msg::RemovedFromQueue(&removed.title));
                if self.queue.is_empty() {
                    self.queue_state.select(None);
                } else if idx >= self.queue.len() {
                    self.queue_state.select(Some(self.queue.len() - 1));
                }
            }
        } else if self.active_view == ActiveView::Home && !self.tracks.is_empty() {
            let removed = self.tracks.remove(self.selected_index);
            let _ = self.store.delete_track(&removed.id);
            if self.tracks.is_empty() {
                self.selected_index = 0;
                self.list_state.select(None);
            } else if self.selected_index >= self.tracks.len() {
                self.selected_index = self.tracks.len() - 1;
                self.list_state.select(Some(self.selected_index));
            }
            self.say(Msg::RemovedFromRecent(&removed.title));
        }
    }

    pub fn clear_current_list(&mut self) {
        if self.active_view == ActiveView::Queue {
            self.queue.clear();
            self.queue_state.select(None);
            self.say(Msg::QueueCleared);
        } else if self.active_view == ActiveView::Home {
            let _ = self.store.clear_all_tracks();
            self.tracks.clear();
            self.selected_index = 0;
            self.list_state.select(None);
            self.say(Msg::RecentCleared);
        }
    }
}
