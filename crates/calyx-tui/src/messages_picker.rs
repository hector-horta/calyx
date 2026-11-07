//! Status-line messages about editing playlists: the 'm' picker (add to /
//! create a playlist) and 'd' inside an opened playlist (remove a song).

use calyx_core::Language;

pub enum PickerMsg<'a> {
    PickPlaylist(&'a str),
    CreatingPlaylist { name: &'a str, title: &'a str },
    PlaylistCreated { name: &'a str, title: &'a str },
    PlaylistCreateError { name: &'a str, err: &'a str },
    AddingToPlaylist { title: &'a str, playlist: &'a str },
    AddedToPlaylist { title: &'a str, playlist: &'a str },
    AddToPlaylistError { playlist: &'a str, err: &'a str },
    ConfirmRemove { title: &'a str, playlist: &'a str },
    Removing { title: &'a str, playlist: &'a str },
    Removed { title: &'a str, playlist: &'a str },
    RemoveError { playlist: &'a str, err: &'a str },
    NotEditable(&'a str),
    NoLibraryId(&'a str),
}

impl<'a> From<PickerMsg<'a>> for crate::messages::Msg<'a> {
    fn from(msg: PickerMsg<'a>) -> Self {
        crate::messages::Msg::Picker(msg)
    }
}

impl PickerMsg<'_> {
    pub fn text(&self, lang: Language) -> String {
        use Language::{En, Es};
        match (self, lang) {
            (PickerMsg::PickPlaylist(title), Es) => {
                format!("Elige una playlist para '{title}' (Enter agrega, Esc cancela).")
            }
            (PickerMsg::PickPlaylist(title), En) => {
                format!("Choose a playlist for '{title}' (Enter adds, Esc cancels).")
            }
            (PickerMsg::CreatingPlaylist { name, title }, Es) => {
                format!("Creando la playlist '{name}' con '{title}'...")
            }
            (PickerMsg::CreatingPlaylist { name, title }, En) => {
                format!("Creating playlist '{name}' with '{title}'...")
            }
            (PickerMsg::PlaylistCreated { name, title }, Es) => {
                format!("Playlist '{name}' creada con '{title}'.")
            }
            (PickerMsg::PlaylistCreated { name, title }, En) => {
                format!("Playlist '{name}' created with '{title}'.")
            }
            (PickerMsg::PlaylistCreateError { name, err }, Es) => {
                format!("No se pudo crear '{name}': {err}")
            }
            (PickerMsg::PlaylistCreateError { name, err }, En) => {
                format!("Could not create '{name}': {err}")
            }
            (PickerMsg::AddingToPlaylist { title, playlist }, Es) => {
                format!("Agregando '{title}' a '{playlist}'...")
            }
            (PickerMsg::AddingToPlaylist { title, playlist }, En) => {
                format!("Adding '{title}' to '{playlist}'...")
            }
            (PickerMsg::AddedToPlaylist { title, playlist }, Es) => {
                format!("'{title}' agregada a la playlist '{playlist}'.")
            }
            (PickerMsg::AddedToPlaylist { title, playlist }, En) => {
                format!("'{title}' added to playlist '{playlist}'.")
            }
            (PickerMsg::AddToPlaylistError { playlist, err }, Es) => {
                format!("No se pudo agregar a '{playlist}': {err}")
            }
            (PickerMsg::AddToPlaylistError { playlist, err }, En) => {
                format!("Could not add to '{playlist}': {err}")
            }
            (PickerMsg::ConfirmRemove { title, playlist }, Es) => {
                format!("¿Quitar '{title}' de '{playlist}'? Presiona d otra vez")
            }
            (PickerMsg::ConfirmRemove { title, playlist }, En) => {
                format!("Remove '{title}' from '{playlist}'? Press d again")
            }
            (PickerMsg::Removing { title, playlist }, Es) => {
                format!("Quitando '{title}' de '{playlist}'...")
            }
            (PickerMsg::Removing { title, playlist }, En) => {
                format!("Removing '{title}' from '{playlist}'...")
            }
            (PickerMsg::Removed { title, playlist }, Es) => {
                format!("'{title}' quitada de '{playlist}'.")
            }
            (PickerMsg::Removed { title, playlist }, En) => {
                format!("'{title}' removed from '{playlist}'.")
            }
            (PickerMsg::RemoveError { playlist, err }, Es) => {
                format!("No se pudo quitar de '{playlist}': {err}")
            }
            (PickerMsg::RemoveError { playlist, err }, En) => {
                format!("Could not remove from '{playlist}': {err}")
            }
            (PickerMsg::NotEditable(p), Es) => {
                format!("La playlist '{p}' no es tuya: no se puede editar.")
            }
            (PickerMsg::NotEditable(p), En) => {
                format!("Playlist '{p}' is not yours: it cannot be edited.")
            }
            (PickerMsg::NoLibraryId(t), Es) => {
                format!("'{t}' no tiene id de biblioteca: no se puede quitar.")
            }
            (PickerMsg::NoLibraryId(t), En) => {
                format!("'{t}' has no library id: it cannot be removed.")
            }
        }
    }
}
