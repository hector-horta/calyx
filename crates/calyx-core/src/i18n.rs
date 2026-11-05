use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Es,
    En,
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Language::Es => serializer.serialize_str("es"),
            Language::En => serializer.serialize_str("en"),
        }
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "es" => Ok(Language::Es),
            "en" => Ok(Language::En),
            _ => Err(serde::de::Error::custom("Invalid language code")),
        }
    }
}

impl Language {
    pub fn toggle(self) -> Self { match self { Self::Es => Self::En, Self::En => Self::Es } }
    pub fn code(&self) -> &'static str { match self { Self::Es => "es", Self::En => "en" } }
    pub fn name(&self) -> &'static str { match self { Self::Es => "Español", Self::En => "English" } }
    pub fn detect_system() -> Self {
        for k in ["LANG", "LC_ALL", "LC_MESSAGES"] {
            if let Ok(v) = std::env::var(k) {
                if v.to_lowercase().starts_with("es") { return Self::Es; }
            }
        }
        Self::En
    }
    pub fn nav_home(&self) -> &'static str { match self { Self::Es => "Inicio", Self::En => "Home" } }
    pub fn nav_new(&self) -> &'static str { match self { Self::Es => "Novedades", Self::En => "New" } }
    pub fn nav_radio(&self) -> &'static str { match self { Self::Es => "Radio", Self::En => "Radio" } }
    pub fn nav_library(&self) -> &'static str { match self { Self::Es => "Biblioteca", Self::En => "Library" } }
    pub fn nav_playlists(&self) -> &'static str { match self { Self::Es => "Playlists", Self::En => "Playlists" } }
    pub fn nav_search(&self) -> &'static str { match self { Self::Es => "Buscar", Self::En => "Search" } }
    pub fn nav_queue(&self) -> &'static str { match self { Self::Es => "Cola", Self::En => "Queue" } }
    pub fn nav_login(&self) -> &'static str { match self { Self::Es => "Iniciar sesión", Self::En => "Log in" } }
    pub fn nav_help(&self) -> &'static str { match self { Self::Es => "Ayuda", Self::En => "Help" } }
    pub fn nav_lang(&self) -> &'static str { match self { Self::Es => "Idioma", Self::En => "Language" } }
    pub fn key_space(&self) -> &'static str { match self { Self::Es => "Espacio", Self::En => "Space" } }
    pub fn btn_play(&self) -> &'static str { match self { Self::Es => "Reproducir", Self::En => "Play" } }
    pub fn btn_pause(&self) -> &'static str { match self { Self::Es => "Pausa", Self::En => "Pause" } }
    pub fn btn_prev_next(&self) -> &'static str { match self { Self::Es => "Anterior/Siguiente", Self::En => "Prev/Next" } }
    pub fn btn_volume(&self) -> &'static str { match self { Self::Es => "Volumen", Self::En => "Volume" } }
    pub fn btn_open(&self) -> &'static str { match self { Self::Es => "Abrir", Self::En => "Open" } }
    pub fn btn_play_list(&self) -> &'static str { match self { Self::Es => "Reproducir lista", Self::En => "Play playlist" } }
    pub fn btn_enqueue(&self) -> &'static str { match self { Self::Es => "Encolar", Self::En => "Enqueue" } }
    pub fn btn_next_in_queue(&self) -> &'static str { match self { Self::Es => "A continuación", Self::En => "Play next" } }
    pub fn btn_remove_track(&self) -> &'static str { match self { Self::Es => "Quitar", Self::En => "Remove" } }
    pub fn btn_clear_all(&self) -> &'static str { match self { Self::Es => "Limpiar", Self::En => "Clear" } }
    pub fn btn_shuffle(&self) -> &'static str { match self { Self::Es => "Aleatorio", Self::En => "Shuffle" } }
    pub fn btn_repeat(&self) -> &'static str { match self { Self::Es => "Repetir", Self::En => "Repeat" } }
    pub fn btn_repeat_all(&self) -> &'static str { match self { Self::Es => "Repetir: todo", Self::En => "Repeat: all" } }
    pub fn btn_repeat_one(&self) -> &'static str { match self { Self::Es => "Repetir: una", Self::En => "Repeat: one" } }
    pub fn btn_phonetic_dual(&self) -> &'static str { match self { Self::Es => "Fonética: Dual", Self::En => "Lyrics: Dual" } }
    pub fn btn_phonetic_only(&self) -> &'static str { match self { Self::Es => "Fonética: Romaji", Self::En => "Lyrics: Romaji" } }
    pub fn btn_phonetic_off(&self) -> &'static str { match self { Self::Es => "Fonética: Off", Self::En => "Lyrics: Off" } }
    pub fn col_title(&self) -> &'static str { match self { Self::Es => "Título", Self::En => "Title" } }
    pub fn col_artist(&self) -> &'static str { match self { Self::Es => "Artista", Self::En => "Artist" } }
    pub fn col_album(&self) -> &'static str { match self { Self::Es => "Álbum", Self::En => "Album" } }
    pub fn col_duration(&self) -> &'static str { match self { Self::Es => "Dur.", Self::En => "Time" } }
    pub fn lbl_up_next(&self) -> &'static str { match self { Self::Es => "A continuación", Self::En => "Up next" } }
    pub fn lbl_in_queue(&self) -> &'static str { match self { Self::Es => "en cola", Self::En => "in queue" } }
    pub fn msg_no_playlists(&self) -> &'static str { match self { Self::Es => "No hay playlists cargadas. Presiona 5 para sincronizar o u para iniciar sesión.", Self::En => "No playlists loaded. Press 5 to sync or u to log in." } }
    pub fn msg_queue_empty(&self) -> &'static str { match self { Self::Es => "La cola está vacía. Selecciona pistas y presiona a o n.", Self::En => "The queue is empty. Select tracks and press a or n." } }
    pub fn search_placeholder(&self) -> &'static str { match self { Self::Es => " 🔍 Presiona '/' para buscar canciones, artistas o álbumes...", Self::En => " 🔍 Press '/' to search songs, artists or albums..." } }
    pub fn status_playing(&self) -> &'static str { match self { Self::Es => "▶  Reproduciendo", Self::En => "▶  Playing" } }
    pub fn status_paused(&self) -> &'static str { match self { Self::Es => "⏸  En Pausa", Self::En => "⏸  Paused" } }
    pub fn status_buffering(&self) -> &'static str { match self { Self::Es => "⏳  Cargando...", Self::En => "⏳  Buffering..." } }
    pub fn status_stopped(&self) -> &'static str { match self { Self::Es => "⏹  Detenido", Self::En => "⏹  Stopped" } }
    pub fn title_player(&self) -> &'static str { match self { Self::Es => " Reproductor ", Self::En => " Player " } }
    pub fn title_cover(&self) -> &'static str { match self { Self::Es => " Portada ", Self::En => " Cover " } }
    pub fn no_active_playback(&self) -> &'static str { match self { Self::Es => " Sin reproducción activa", Self::En => " No active playback" } }
    pub fn no_lyrics_track(&self) -> &'static str { match self { Self::Es => "♪ Letra no disponible", Self::En => "♪ Lyrics not available" } }
    pub fn no_lyrics(&self) -> &'static str { match self { Self::Es => "  ♪ Reproduce una canción para ver su letra sincronizada en vivo.", Self::En => "  ♪ Play a song to see live synchronized lyrics." } }
    pub fn no_browser_guide(&self) -> &'static str { match self { Self::Es => "No se encontró navegador. Consulta 'Autenticación' en README.", Self::En => "No browser found. See 'Authentication' in README." } }
    pub fn msg_login_safari_alert(&self) -> &'static str { match self { Self::Es => "⚠️ Safari detectado: Haz clic en 'Permitir' cuando macOS te pida confirmación.", Self::En => "⚠️ Safari detected: Click 'Allow' when macOS prompts for permissions." } }
    pub fn msg_login_opening(&self) -> &'static str { match self { Self::Es => "Abriendo Apple Sign-In... Inicia sesión y la sesión se sincronizará automáticamente.", Self::En => "Opening Apple Sign-In... Log in and session will sync automatically." } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        assert_eq!(Language::Es.toggle(), Language::En);
        assert_eq!(Language::En.toggle(), Language::Es);
        assert_eq!(Language::Es.code(), "es");
        assert_eq!(Language::En.code(), "en");
    }
}