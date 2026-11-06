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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_language() {
        assert_eq!(album(Language::Es), "Álbum");
        assert_eq!(album_to_library(Language::En), "Save album");
    }
}
