//! Which messages become a notice and with which tone (see notice.rs), and
//! in which zone of the screen each one is shown (see zones.rs).

use crate::messages::Msg;
use crate::messages_picker::PickerMsg;
use crate::notice::Tone;
use crate::zones::Zone;

impl Msg<'_> {
    /// Where the message is shown: list work in the list frame, the rest
    /// (playback, modes, language, login) in the player box.
    pub fn zone(&self) -> Zone {
        match self {
            Msg::Enqueued { .. }
            | Msg::EnqueuedNext { .. }
            | Msg::RemovedFromQueue(..)
            | Msg::RemovedFromRecent(..)
            | Msg::QueueCleared
            | Msg::RecentCleared
            | Msg::Searching(..)
            | Msg::Loading(..)
            | Msg::ListLoaded { .. }
            | Msg::ListEmpty(..)
            | Msg::ListError { .. }
            | Msg::SyncingPlaylists
            | Msg::PlaylistsSynced(..)
            | Msg::PlaylistsError(..)
            | Msg::LoadingPlaylist(..)
            | Msg::PlaylistOpened { .. }
            | Msg::PlaylistError { .. }
            | Msg::LoadingAlbum(..)
            | Msg::AlbumOpened { .. }
            | Msg::BackToList(..)
            | Msg::AlbumError(..)
            | Msg::AlbumEmpty(..)
            | Msg::ListEnqueued(..)
            | Msg::AddingAlbumToLibrary(..)
            | Msg::AlbumAddedToLibrary(..)
            | Msg::AlbumAddError { .. }
            | Msg::RecentOpened
            | Msg::Picker(..) => Zone::List,
            _ => Zone::Player,
        }
    }

    /// Returns which notices must stand out (None = routine progress/info).
    pub fn tone(&self) -> Option<Tone> {
        match self {
            Msg::Enqueued { .. }
            | Msg::EnqueuedNext { .. }
            | Msg::RemovedFromQueue(..)
            | Msg::RemovedFromRecent(..)
            | Msg::QueueCleared
            | Msg::RecentCleared
            | Msg::ListEnqueued(..)
            | Msg::AlbumAddedToLibrary(..)
            | Msg::LoginOk => Some(Tone::Result),
            Msg::ListError { .. }
            | Msg::PlaylistsError(..)
            | Msg::PlaylistError { .. }
            | Msg::PlaylistPlayFailed(..)
            | Msg::AlbumError(..)
            | Msg::AlbumAddError { .. }
            | Msg::LoginError(..)
            | Msg::AudioError(..) => Some(Tone::Error),
            Msg::Picker(p) => p.tone(),
            _ => None,
        }
    }
}

impl PickerMsg<'_> {
    /// Returns the tone for this picker message.
    pub fn tone(&self) -> Option<Tone> {
        match self {
            PickerMsg::ConfirmRemove { .. } => Some(Tone::Action),
            PickerMsg::PlaylistCreated { .. }
            | PickerMsg::AddedToPlaylist { .. }
            | PickerMsg::Removed { .. } => Some(Tone::Result),
            PickerMsg::PlaylistCreateError { .. }
            | PickerMsg::AddToPlaylistError { .. }
            | PickerMsg::RemoveError { .. }
            | PickerMsg::NotEditable(..)
            | PickerMsg::NoLibraryId(..) => Some(Tone::Error),
            _ => None,
        }
    }
}
