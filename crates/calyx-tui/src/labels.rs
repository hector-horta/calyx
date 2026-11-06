//! Short bottom-bar labels for keys added by the TUI (core's `i18n` holds the older ones).

use calyx_core::Language;

/// Label of the 'o' key (open the album of the selected song).
pub fn album(lang: Language) -> &'static str {
    match lang {
        Language::Es => "Álbum",
        Language::En => "Album",
    }
}

/// Label of the 'f' key (add the album to the library).
pub fn album_to_library(lang: Language) -> &'static str {
    match lang {
        Language::Es => "Guardar álbum",
        Language::En => "Save album",
    }
}

/// Label of the 'm' key (add the selected song to a playlist).
pub fn to_playlist(lang: Language) -> &'static str {
    match lang {
        Language::Es => "A playlist",
        Language::En => "To playlist",
    }
}

/// Texts of the 'm' picker: (title for `song`, add, cancel, loading, none editable).
pub fn picker(
    lang: Language,
    song: &str,
) -> (
    String,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    match lang {
        Language::Es => (
            format!(" Agregar '{song}' a playlist "),
            "Agregar",
            "Cancelar",
            "Cargando tus playlists...",
            "No tienes playlists editables.",
        ),
        Language::En => (
            format!(" Add '{song}' to playlist "),
            "Add",
            "Cancel",
            "Loading your playlists...",
            "You have no editable playlists.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_language() {
        assert_eq!(album(Language::Es), "Álbum");
        assert_eq!(album_to_library(Language::En), "Save album");
    }
}
