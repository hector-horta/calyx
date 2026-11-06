//! Every status-line message of the TUI, in both languages, in one place.
//! Callers build a `Msg` with its data; `text(lang)` renders it.

use calyx_core::{Language, RepeatMode};

/// Which track list a background load filled.
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

pub enum Msg<'a> {
    Enqueued { artist: &'a str, title: &'a str },
    EnqueuedNext { artist: &'a str, title: &'a str },
    RemovedFromQueue(&'a str),
    RemovedFromRecent(&'a str),
    QueueCleared,
    RecentCleared,
    Repeat(RepeatMode),
    Shuffle(bool),
    LanguageSet,
    Searching(&'a str),
    Loading(ListKind),
    ListLoaded { kind: ListKind, count: usize },
    ListEmpty(ListKind),
    ListError { kind: ListKind, err: &'a str },
    SyncingPlaylists,
    PlaylistsSynced(usize),
    PlaylistsError(&'a str),
    LoadingPlaylist(&'a str),
    PlaylistOpened { name: &'a str, count: usize },
    PlaylistError { name: &'a str, err: &'a str },
    PlayingPlaylist { name: &'a str, count: usize },
    PlaylistPlayFailed(&'a str),
    LoginOk,
    LoginError(&'a str),
    HelpOpened,
    LoadingAudio { artist: &'a str, title: &'a str },
    PlayingList,
    RecentOpened,
    AudioError(&'a str),
    SyncingLyrics,
    HintCache,
    HintFull,
    HintPreview(Option<&'a str>),
}

impl Msg<'_> {
    pub fn text(&self, lang: Language) -> String {
        use Language::{En, Es};
        match (self, lang) {
            (Msg::Enqueued { artist, title }, Es) => {
                format!("Encolada al final: {artist} — {title}")
            }
            (Msg::Enqueued { artist, title }, En) => {
                format!("Added to the end of the queue: {artist} — {title}")
            }
            (Msg::EnqueuedNext { artist, title }, Es) => {
                format!("Encolada a continuación: {artist} — {title}")
            }
            (Msg::EnqueuedNext { artist, title }, En) => {
                format!("Playing next: {artist} — {title}")
            }
            (Msg::RemovedFromQueue(title), Es) => format!("Eliminada de la cola: {title}"),
            (Msg::RemovedFromQueue(title), En) => format!("Removed from the queue: {title}"),
            (Msg::RemovedFromRecent(title), Es) => format!("Eliminada de Recientes: {title}"),
            (Msg::RemovedFromRecent(title), En) => format!("Removed from Recent: {title}"),
            (Msg::QueueCleared, Es) => "Cola de reproducción vaciada.".to_string(),
            (Msg::QueueCleared, En) => "Queue cleared.".to_string(),
            (Msg::RecentCleared, Es) => "Historial de Recientes vaciado.".to_string(),
            (Msg::RecentCleared, En) => "Recent history cleared.".to_string(),
            (Msg::Repeat(mode), Es) => format!(
                "Modo repetición: {}",
                match mode {
                    RepeatMode::Off => "desactivado",
                    RepeatMode::All => "todo",
                    RepeatMode::One => "una",
                }
            ),
            (Msg::Repeat(mode), En) => format!(
                "Repeat mode: {}",
                match mode {
                    RepeatMode::Off => "off",
                    RepeatMode::All => "all",
                    RepeatMode::One => "one",
                }
            ),
            (Msg::Shuffle(true), Es) => "Aleatorio: activado".to_string(),
            (Msg::Shuffle(true), En) => "Shuffle: on".to_string(),
            (Msg::Shuffle(false), Es) => "Aleatorio: desactivado".to_string(),
            (Msg::Shuffle(false), En) => "Shuffle: off".to_string(),
            (Msg::LanguageSet, Es) => "Idioma: Español ('L' para cambiar)".to_string(),
            (Msg::LanguageSet, En) => "Language: English ('L' to switch)".to_string(),
            (Msg::Searching(q), Es) => format!("Buscando '{q}'..."),
            (Msg::Searching(q), En) => format!("Searching '{q}'..."),
            (Msg::Loading(kind), Es) => format!("Cargando {}...", kind.name(lang)),
            (Msg::Loading(kind), En) => format!("Loading {}...", kind.name(lang)),
            (Msg::ListLoaded { kind, count }, Es) => {
                format!("{}: {count} pistas.", kind.name(lang))
            }
            (Msg::ListLoaded { kind, count }, En) => {
                format!("{}: {count} tracks.", kind.name(lang))
            }
            (Msg::ListEmpty(kind), Es) => format!("{}: sin canciones.", kind.name(lang)),
            (Msg::ListEmpty(kind), En) => format!("{}: no songs.", kind.name(lang)),
            (Msg::ListError { kind, err }, Es) => format!("Error en {}: {err}", kind.name(lang)),
            (Msg::ListError { kind, err }, En) => format!("{} failed: {err}", kind.name(lang)),
            (Msg::SyncingPlaylists, Es) => {
                "Sincronizando tus playlists de Apple Music...".to_string()
            }
            (Msg::SyncingPlaylists, En) => "Syncing your Apple Music playlists...".to_string(),
            (Msg::PlaylistsSynced(n), Es) => {
                format!("Playlists sincronizadas: {n} (Enter para abrir).")
            }
            (Msg::PlaylistsSynced(n), En) => format!("Playlists synced: {n} (Enter to open)."),
            (Msg::PlaylistsError(err), Es) => format!("Error sincronizando playlists: {err}"),
            (Msg::PlaylistsError(err), En) => format!("Could not sync playlists: {err}"),
            (Msg::LoadingPlaylist(name), Es) => format!("⏳ Cargando playlist '{name}'..."),
            (Msg::LoadingPlaylist(name), En) => format!("⏳ Loading playlist '{name}'..."),
            (Msg::PlaylistOpened { name, count }, Es) => format!(
                "Playlist '{name}' cargada ({count} canciones). Presiona 'p' para reproducirla entera."
            ),
            (Msg::PlaylistOpened { name, count }, En) => {
                format!("Playlist '{name}' loaded ({count} songs). Press 'p' to play it all.")
            }
            (Msg::PlaylistError { name, err }, Es) => {
                format!("Error cargando la playlist '{name}': {err}")
            }
            (Msg::PlaylistError { name, err }, En) => {
                format!("Could not load playlist '{name}': {err}")
            }
            (Msg::PlayingPlaylist { name, count }, Es) => {
                format!("Reproduciendo '{name}' ({count} pistas)...")
            }
            (Msg::PlayingPlaylist { name, count }, En) => {
                format!("Playing '{name}' ({count} tracks)...")
            }
            (Msg::PlaylistPlayFailed(name), Es) => {
                format!("No se pudo reproducir la playlist '{name}'.")
            }
            (Msg::PlaylistPlayFailed(name), En) => format!("Could not play playlist '{name}'."),
            (Msg::LoginOk, Es) => "Sesión de Apple Music verificada.".to_string(),
            (Msg::LoginOk, En) => "Apple Music session verified.".to_string(),
            (Msg::LoginError(err), Es) => format!("Error de login en Apple: {err}"),
            (Msg::LoginError(err), En) => format!("Apple login failed: {err}"),
            (Msg::HelpOpened, Es) => "Ayuda de atajos (1-5 o Tab para volver).".to_string(),
            (Msg::HelpOpened, En) => "Keyboard help (1-5 or Tab to go back).".to_string(),
            (Msg::LoadingAudio { artist, title }, Es) => {
                format!("⏳ Cargando audio: {artist} — {title}...")
            }
            (Msg::LoadingAudio { artist, title }, En) => {
                format!("⏳ Loading audio: {artist} — {title}...")
            }
            (Msg::PlayingList, Es) => "Reproduciendo la lista actual...".to_string(),
            (Msg::PlayingList, En) => "Playing the current list...".to_string(),
            (Msg::RecentOpened, Es) => "1 — Recientes: historial de búsquedas ('/').".to_string(),
            (Msg::RecentOpened, En) => "1 — Recent: search history ('/').".to_string(),
            (Msg::AudioError(err), Es) => format!("Error de audio: {err}"),
            (Msg::AudioError(err), En) => format!("Audio error: {err}"),
            (Msg::SyncingLyrics, Es) => "⏳ Sincronizando letra...".to_string(),
            (Msg::SyncingLyrics, En) => "⏳ Syncing lyrics...".to_string(),
            (Msg::HintCache, Es) => "[AAC 256k | Caché]".to_string(),
            (Msg::HintCache, En) => "[AAC 256k | Cached]".to_string(),
            (Msg::HintFull, _) => "[AAC 256k]".to_string(),
            (Msg::HintPreview(Some(note)), _) => format!("[Preview 30s | {note}]"),
            (Msg::HintPreview(None), Es) => {
                "[Preview 30s — Presiona 'u' para iniciar sesión en Apple]".to_string()
            }
            (Msg::HintPreview(None), En) => {
                "[Preview 30s — press 'u' to log in to Apple]".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_follow_the_language() {
        let msg = Msg::PlaylistOpened {
            name: "Mix",
            count: 3,
        };
        assert!(
            msg.text(Language::Es)
                .starts_with("Playlist 'Mix' cargada (3 canciones)")
        );
        assert!(
            msg.text(Language::En)
                .starts_with("Playlist 'Mix' loaded (3 songs)")
        );
    }

    #[test]
    fn list_kind_is_localized_inside_messages() {
        let msg = Msg::ListLoaded {
            kind: ListKind::Library,
            count: 50,
        };
        assert_eq!(msg.text(Language::Es), "Biblioteca: 50 pistas.");
        assert_eq!(msg.text(Language::En), "Library: 50 tracks.");
    }
}
