//! 'd' inside an opened playlist: remove the selected song. The first 'd' asks,
//! a second 'd' on the same song within a few seconds confirms.

use crate::activity::Job;
use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use crate::messages_picker::PickerMsg;
use calyx_core::StreamingBackend;
use std::time::{Duration, Instant};

/// A second 'd' within this time confirms the removal.
const CONFIRM_WINDOW: Duration = Duration::from_secs(4);

impl TuiApp {
    /// An editable playlist is the visible list ('d' removes from it).
    pub fn can_remove_from_open_playlist(&self) -> bool {
        self.active_view == crate::app::ActiveView::Library
            && !self.showing_album()
            && self.open_playlist.as_ref().is_some_and(|p| p.can_edit)
    }

    /// Removes the selected song from the open playlist after confirmation.
    pub fn remove_selected_from_playlist(&mut self) {
        let Some(playlist) = self.open_playlist.clone() else {
            return;
        };
        if self.showing_album() {
            return;
        }
        if !playlist.can_edit {
            self.say(PickerMsg::NotEditable(&playlist.name));
            return;
        }

        let Some(track) = self.tracks.get(self.selected_index).cloned() else {
            return;
        };

        let Some(song_id) = track
            .hls_manifest_url
            .clone()
            .filter(|id| id.starts_with("i."))
        else {
            self.say(PickerMsg::NoLibraryId(&track.title));
            return;
        };

        let confirmed = matches!(&self.pending_remove, Some((id, at)) if *id == song_id && at.elapsed() < CONFIRM_WINDOW);

        if !confirmed {
            self.pending_remove = Some((song_id, Instant::now()));
            self.say(PickerMsg::ConfirmRemove {
                title: &track.title,
                playlist: &playlist.name,
            });
            return;
        }

        self.pending_remove = None;
        self.say(PickerMsg::Removing {
            title: &track.title,
            playlist: &playlist.name,
        });
        self.activity.start(Job::Write);

        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .remove_from_playlist(&playlist.id, &song_id)
                .await
                .map_err(|e| e.to_string());
            BgEvent::RemovedFromPlaylist {
                playlist_id: playlist.id,
                playlist: playlist.name,
                song_id,
                title: track.title,
                res,
            }
        });
    }

    /// Handles Apple's answer to a playlist removal.
    pub(crate) fn on_removed_from_playlist(
        &mut self,
        playlist_id: &str,
        playlist: &str,
        song_id: &str,
        title: &str,
        res: Result<(), String>,
    ) {
        if let Err(err) = res {
            self.say(PickerMsg::RemoveError {
                playlist,
                err: &err,
            });
            return;
        }
        self.say(PickerMsg::Removed { title, playlist });
        let still_open = self
            .open_playlist
            .as_ref()
            .is_some_and(|p| p.id == playlist_id)
            && !self.showing_album();
        if !still_open {
            return;
        }

        self.tracks
            .retain(|t| t.hls_manifest_url.as_deref() != Some(song_id));
        if self.tracks.is_empty() {
            self.selected_index = 0;
            self.list_state.select(None);
        } else {
            self.selected_index = self.selected_index.min(self.tracks.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }
}
