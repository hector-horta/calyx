//! Status-line messages of the 'm' playlist picker (add to / create a playlist).

use calyx_core::Language;

pub enum PickerMsg<'a> {
    PickPlaylist(&'a str),
    CreatingPlaylist { name: &'a str, title: &'a str },
    PlaylistCreated { name: &'a str, title: &'a str },
    PlaylistCreateError { name: &'a str, err: &'a str },
    AddingToPlaylist { title: &'a str, playlist: &'a str },
    AddedToPlaylist { title: &'a str, playlist: &'a str },
    AddToPlaylistError { playlist: &'a str, err: &'a str },
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
        }
    }
}
