//! 'm': choose one of the user's editable playlists and add the selected song to it.
//! While the picker is open it receives every key (see `main.rs`).

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use crate::messages_picker::PickerMsg;
use calyx_core::{StreamingBackend, Track, UserPlaylist};
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

pub struct PlaylistPicker {
    /// The song to add.
    pub track: Track,
    /// Selected row: 0 is "new playlist", row i + 1 is `editable_playlists()[i]`.
    pub state: ListState,
    /// Name being typed for a new playlist (the popup shows a text field).
    pub naming: Option<String>,
}

/// Longest name the text field accepts (the core validates it again).
const MAX_NAME_CHARS: usize = 100;

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
        self.say(PickerMsg::PickPlaylist(&track.title));
        // Start on the first real playlist so 'm' + Enter stays one gesture.
        let first = usize::from(!self.editable_playlists().is_empty());
        self.picker = Some(PlaylistPicker {
            track,
            state: ListState::default().with_selected(Some(first)),
            naming: None,
        });
    }

    /// Playlists arrived while the picker was open on "new playlist": move to the
    /// first real one, so 'm' + Enter behaves the same as when they were loaded.
    pub(crate) fn select_first_playlist_in_picker(&mut self) {
        let has_playlists = !self.editable_playlists().is_empty();
        if let Some(p) = self.picker.as_mut()
            && p.naming.is_none()
            && p.state.selected() == Some(0)
            && has_playlists
        {
            p.state.select(Some(1));
        }
    }

    /// Keys while the picker is open: move, add with Enter, close with Esc.
    pub fn picker_key(&mut self, code: KeyCode) {
        if self.picker.as_ref().is_some_and(|p| p.naming.is_some()) {
            self.naming_key(code);
            return;
        }
        let len = self.editable_playlists().len() + 1; // + the "new playlist" row
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
            KeyCode::Enter if current == 0 => picker.naming = Some(String::new()),
            KeyCode::Char('n') => picker.naming = Some(String::new()),
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
        let Some(playlist) = index
            .checked_sub(1)
            .and_then(|i| self.editable_playlists().get(i).map(|p| (*p).clone()))
        else {
            return; // still loading, or nothing editable: keep the picker open
        };
        let Some(picker) = self.picker.take() else {
            return;
        };
        let track = picker.track;
        self.say(PickerMsg::AddingToPlaylist {
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

    /// Creates a new playlist with the given name and adds the selected song to it.
    fn confirm_new_playlist(&mut self) {
        let name = self
            .picker
            .as_ref()
            .and_then(|p| p.naming.as_deref())
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            return;
        }
        let Some(picker) = self.picker.take() else {
            return;
        };
        let track = picker.track;
        self.say(PickerMsg::CreatingPlaylist {
            name: &name,
            title: &track.title,
        });
        self.activity.start(crate::activity::Job::Write);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .create_playlist_with(&name, &track)
                .await
                .map_err(|e| e.to_string());
            BgEvent::PlaylistCreated {
                title: track.title,
                name,
                res,
            }
        });
    }

    /// Keys while typing a new playlist name: edit it, Enter creates, Esc back to the list.
    fn naming_key(&mut self, code: KeyCode) {
        let Some(name) = self.picker.as_mut().and_then(|p| p.naming.as_mut()) else {
            return;
        };
        match code {
            KeyCode::Char(c) if !c.is_control() && name.chars().count() < MAX_NAME_CHARS => {
                name.push(c)
            }
            KeyCode::Backspace => {
                name.pop();
            }
            KeyCode::Enter => self.confirm_new_playlist(),
            KeyCode::Esc => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.naming = None;
                }
            }
            _ => {}
        }
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
