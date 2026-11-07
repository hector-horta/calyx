//! Short bottom-bar labels for keys added by the TUI (core's `i18n` holds the older ones).

use crate::app::ActiveView;
use calyx_core::Language;

/// Label of the 'o' key (open the album of the selected song).
pub fn album(lang: Language) -> &'static str {
    match lang {
        Language::Es => "Álbum",
        Language::En => "Album",
    }
}

/// Label of the 'o' key while an album is shown: where it goes back to.
pub fn back_to(lang: Language, view: ActiveView) -> &'static str {
    match (lang, view) {
        (Language::Es, ActiveView::New) => "← Novedades",
        (Language::Es, ActiveView::Radio) => "← Radio",
        (Language::Es, ActiveView::Library) => "← Biblioteca",
        (Language::Es, ActiveView::Queue) => "← Cola",
        (Language::Es, ActiveView::Playlists) => "← Playlists",
        (Language::Es, _) => "← Inicio",
        (Language::En, ActiveView::New) => "← New",
        (Language::En, ActiveView::Radio) => "← Radio",
        (Language::En, ActiveView::Library) => "← Library",
        (Language::En, ActiveView::Queue) => "← Queue",
        (Language::En, ActiveView::Playlists) => "← Playlists",
        (Language::En, _) => "← Home",
    }
}

/// Name of a view, as on its tab.
pub fn view_name(lang: Language, view: ActiveView) -> &'static str {
    match view {
        ActiveView::Home | ActiveView::Help => lang.nav_home(),
        ActiveView::New => lang.nav_new(),
        ActiveView::Radio => lang.nav_radio(),
        ActiveView::Library => lang.nav_library(),
        ActiveView::Playlists => lang.nav_playlists(),
        ActiveView::Queue => lang.nav_queue(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_language() {
        assert_eq!(album(Language::Es), "Álbum");
        assert_eq!(album_to_library(Language::En), "Save album");
        assert_eq!(back_to(Language::Es, ActiveView::Library), "← Biblioteca");
        assert_eq!(view_name(Language::Es, ActiveView::Library), "Biblioteca");
    }
}
