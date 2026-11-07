//! Texts of the 'm' playlist picker, in both languages.

use calyx_core::Language;

/// Every text the picker popup shows; built once per frame.
pub struct PickerTexts {
    pub title: String,
    pub new_title: String,
    pub new_row: &'static str,
    pub add: &'static str,
    pub new_key: &'static str,
    pub cancel: &'static str,
    pub loading: &'static str,
    pub none: &'static str,
    pub name_label: &'static str,
    pub create: &'static str,
    pub back: &'static str,
}

pub fn picker_texts(lang: Language, song: &str) -> PickerTexts {
    match lang {
        Language::Es => PickerTexts {
            title: format!(" Agregar '{song}' a playlist "),
            new_title: format!(" Nueva playlist para '{song}' "),
            new_row: "➕ Nueva playlist…",
            add: "Agregar",
            new_key: "Nueva",
            cancel: "Cancelar",
            loading: "Cargando tus playlists...",
            none: "No tienes playlists editables.",
            name_label: "Nombre: ",
            create: "Crear",
            back: "Volver",
        },
        Language::En => PickerTexts {
            title: format!(" Add '{song}' to playlist "),
            new_title: format!(" New playlist for '{song}' "),
            new_row: "➕ New playlist…",
            add: "Add",
            new_key: "New",
            cancel: "Cancel",
            loading: "Loading your playlists...",
            none: "You have no editable playlists.",
            name_label: "Name: ",
            create: "Create",
            back: "Back",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_follow_the_language() {
        assert_eq!(
            picker_texts(Language::Es, "X").new_row,
            "➕ Nueva playlist…"
        );
        assert_eq!(
            picker_texts(Language::En, "X").new_title,
            " New playlist for 'X' "
        );
    }
}
