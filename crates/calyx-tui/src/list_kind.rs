//! Which track list a background load filled, with its localized name.

use calyx_core::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    Search,
    NewReleases,
    Radio,
    Library,
}

impl ListKind {
    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Search, Language::Es) => "Búsqueda",
            (Self::Search, Language::En) => "Search",
            (Self::NewReleases, Language::Es) => "Novedades",
            (Self::NewReleases, Language::En) => "New releases",
            (Self::Radio, Language::Es) => "Estaciones de radio",
            (Self::Radio, Language::En) => "Radio stations",
            (Self::Library, Language::Es) => "Biblioteca",
            (Self::Library, Language::En) => "Library",
        }
    }
}
