//! 'm': choose one of the user's editable playlists and add the selected song to it.
//! While the picker is open it receives every key (see `main.rs`).

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use crate::messages::Msg;
use calyx_core::{StreamingBackend, Track, UserPlaylist};
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

pub struct PlaylistPicker {
    /// The song to add.
    pub track: Track,
    /// Selection inside `editable_playlists()`.
    pub state: ListState,
}

/// Next index in a list of `len` items, wrapping around in both directions.
fn step(current: usize, len: usize, down: bool) -> usize {
    match (len, down) {
        (0, _) => 0,
        (_, true) => (current + 1) % len,
        (_, false) => current.checked_sub(1).unwrap_or(len - 1),
    }
}

impl TuiApp {
    /// Playlists the user can add songs to, in library order.
    pub fn editable_playlists(&self) -> Vec<&UserPlaylist> {
        self.playlists.iter().filter(|p| p.can_edit).collect()
    }

    /// 'm' on a track list: open the picker (loading the playlists if needed).
    pub fn open_playlist_picker(&mut self) {
        let Some(track) = self.selected_track() else {
            return;
        };
        if self.playlists.is_empty() {
            self.fetch_playlists();
        }
        self.say(Msg::PickPlaylist(&track.title));
        self.picker = Some(PlaylistPicker {
            track,
            state: ListState::default().with_selected(Some(0)),
        });
    }

    /// Keys while the picker is open: move, add with Enter, close with Esc.
    pub fn picker_key(&mut self, code: KeyCode) {
        let len = self.editable_playlists().len();
        let Some(picker) = self.picker.as_mut() else {
            return;
        };
        let current = picker.state.selected().unwrap_or(0);
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                picker.state.select(Some(step(current, len, true)))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                picker.state.select(Some(step(current, len, false)))
            }
            KeyCode::Enter => self.confirm_picker(),
            KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('q') => self.picker = None,
            _ => {}
        }
    }

    fn confirm_picker(&mut self) {
        let index = self
            .picker
            .as_ref()
            .and_then(|p| p.state.selected())
            .unwrap_or(0);
        let Some(playlist) = self.editable_playlists().get(index).map(|p| (*p).clone()) else {
            return; // still loading, or nothing editable: keep the picker open
        };
        let Some(picker) = self.picker.take() else {
            return;
        };
        let track = picker.track;
        self.say(Msg::AddingToPlaylist {
            title: &track.title,
            playlist: &playlist.name,
        });
        self.activity.start(crate::activity::Job::Write);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .add_to_playlist(&playlist.id, &track)
                .await
                .map_err(|e| e.to_string());
            BgEvent::AddedToPlaylist {
                title: track.title,
                playlist: playlist.name,
                res,
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::step;

    #[test]
    fn selection_wraps_both_ways() {
        assert_eq!(step(0, 3, true), 1);
        assert_eq!(step(2, 3, true), 0);
        assert_eq!(step(0, 3, false), 2);
        assert_eq!(step(1, 3, false), 0);
        assert_eq!(step(0, 0, true), 0);
    }
}
