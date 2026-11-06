//! Results of background work (network, decoding) delivered to the UI thread.
//! Handlers only touch app state; nothing here awaits.

use calyx_core::{AudioSource, DecodedTrack, LyricLine, ResolvedAudio, Track, UserPlaylist};
use crate::app::{ActiveView, TuiApp};
use crate::artwork::TerminalCoverArt;

pub enum BgEvent {
    AudioReady(Track, Result<ResolvedAudio, String>, Vec<LyricLine>),
    AudioDecoded(Track, Result<DecodedTrack, String>, Vec<LyricLine>, String),
    ArtworkReady(String, TerminalCoverArt),
    /// Search, curated sections and the user library.
    TracksReady { view: ActiveView, label: &'static str, res: Result<Vec<Track>, String> },
    PlaylistsReady(Result<Vec<UserPlaylist>, String>),
    /// A playlist opened with Enter (browse its tracks).
    PlaylistOpened(String, Result<Vec<Track>, String>),
    /// A playlist started with 'p' (queue all and play).
    PlaylistReady(String, Result<Vec<Track>, String>),
    LoginDone(Result<(), String>),
}

/// Quality hint shown in the status line for a resolved buffer.
fn audio_hint(audio: &ResolvedAudio) -> String {
    match (audio.source, audio.note.as_deref()) {
        (AudioSource::Cache, _) => "[AAC 256k | Caché]".to_string(),
        (AudioSource::FullStream, _) => "[AAC 256k]".to_string(),
        (AudioSource::Preview, Some(note)) => format!("[Preview 30s | {note}]"),
        (AudioSource::Preview, None) => "[Preview 30s — Presiona 'u' para Login Apple]".to_string(),
    }
}

impl TuiApp {
    pub fn poll_bg_events(&mut self) {
        while let Ok(ev) = self.bg_rx.try_recv() {
            self.handle_bg_event(ev);
        }
    }

    fn is_current(&self, track_id: &str) -> bool {
        self.current_track.as_ref().map(|t| t.id == track_id).unwrap_or(true)
    }

    fn show_tracks(&mut self, tracks: Vec<Track>, view: ActiveView) {
        self.tracks = tracks;
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = view;
    }

    fn handle_bg_event(&mut self, ev: BgEvent) {
        match ev {
            BgEvent::AudioReady(track, res, lyrics) if self.is_current(&track.id) => match res {
                Ok(audio) => {
                    let hint = audio_hint(&audio);
                    self.spawn_decode(track, audio.bytes, lyrics, hint);
                }
                Err(e) => {
                    self.current_lyrics = lyrics;
                    self.status_message = e;
                }
            },
            BgEvent::AudioReady(..) => {}
            BgEvent::AudioDecoded(track, res, lyrics, hint) => self.on_audio_decoded(track, res, lyrics, hint),
            BgEvent::ArtworkReady(tid, art) if self.is_current(&tid) => self.current_artwork = Some(art),
            BgEvent::ArtworkReady(..) => {}
            BgEvent::TracksReady { view, label, res: Ok(list) } => {
                // An empty library sync keeps whatever is on screen.
                if view == ActiveView::Library && list.is_empty() {
                    self.status_message = format!("{label}: sin canciones.");
                    return;
                }
                if view == ActiveView::Home {
                    for t in &list { let _ = self.store.upsert_track(t); }
                }
                self.status_message = format!("{label}: {} pistas.", list.len());
                self.show_tracks(list, view);
            }
            BgEvent::TracksReady { label, res: Err(e), .. } => self.status_message = format!("Error en {label}: {e}"),
            BgEvent::PlaylistsReady(Ok(playlists)) => {
                self.status_message = format!("Playlists sincronizadas: {} listas encontradas (Enter para abrir).", playlists.len());
                self.playlist_state.select(if playlists.is_empty() { None } else { Some(0) });
                self.playlists = playlists;
            }
            BgEvent::PlaylistsReady(Err(e)) => self.status_message = format!("Error sincronizando playlists: {e}"),
            BgEvent::PlaylistOpened(name, Ok(list)) => {
                self.status_message = format!("Playlist '{name}' cargada ({} canciones). Presiona 'p' para reproducir toda.", list.len());
                self.show_tracks(list, ActiveView::Library);
            }
            BgEvent::PlaylistOpened(name, Err(e)) => self.status_message = format!("Error cargando playlist '{name}': {e}"),
            BgEvent::PlaylistReady(name, Ok(list)) if !list.is_empty() => {
                self.queue.clear();
                for t in &list { self.queue.enqueue_back(t.clone()); }
                self.show_tracks(list, ActiveView::Library);
                self.status_message = format!("Reproduciendo '{name}' ({} pistas)...", self.queue.len());
                self.play_next();
            }
            BgEvent::PlaylistReady(name, _) => self.status_message = format!("No se pudo reproducir playlist '{name}'."),
            BgEvent::LoginDone(res) => {
                self.login_in_progress = false;
                self.status_message = match res {
                    Ok(()) => "Sesión Apple Music verificada.".to_string(),
                    Err(e) => format!("Error login Apple: {e}"),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audio(source: AudioSource, note: Option<&str>) -> ResolvedAudio {
        ResolvedAudio { bytes: Vec::new(), source, note: note.map(str::to_string) }
    }

    #[test]
    fn hint_reflects_where_audio_came_from() {
        assert_eq!(audio_hint(&audio(AudioSource::Cache, None)), "[AAC 256k | Caché]");
        assert_eq!(audio_hint(&audio(AudioSource::FullStream, None)), "[AAC 256k]");
        assert_eq!(audio_hint(&audio(AudioSource::Preview, Some("HTTP 403"))), "[Preview 30s | HTTP 403]");
        assert!(audio_hint(&audio(AudioSource::Preview, None)).contains("Login"));
    }
}
