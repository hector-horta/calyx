//! Playback control. How audio is obtained (cache, DRM stream, preview) lives
//! behind the streaming facade; this module only schedules it.

use calyx_core::{StreamingBackend, Track};
use crate::app::{ActiveView, TuiApp};
use crate::artwork::TerminalCoverArt;
use crate::bg_events::BgEvent;

impl TuiApp {
    pub fn play_track(&mut self, track: Track) {
        if let Some(old) = self.play_task.take() { old.abort(); }
        self.audio.stop();
        let seed = format!("{}:{}:{}:{}", track.id, track.artist, track.album, track.title);
        self.current_artwork = Some(TerminalCoverArt::from_jpeg_or_seed(None, &seed));
        self.current_lyrics = vec![calyx_core::LyricLine::new(0, "⏳ Sincronizando letra...")];
        self.current_track = Some(track.clone());
        self.status_message = format!("⏳ Cargando audio en 2do plano: {} — {}...", track.artist, track.title);

        let (backend, tx) = (self.backend.clone(), self.bg_tx.clone());
        self.play_task = Some(tokio::spawn(async move {
            let track = backend.complete_track(track).await;
            if let Some(url) = track.artwork_url.clone() {
                let (backend, tx, id) = (backend.clone(), tx.clone(), track.id.clone());
                tokio::spawn(async move {
                    if let Ok(bytes) = backend.fetch_artwork(&url).await {
                        let _ = tx.send(BgEvent::ArtworkReady(id, TerminalCoverArt::from_jpeg_or_seed(Some(&bytes), &seed)));
                    }
                });
            }
            let (audio, lyrics) = tokio::join!(backend.resolve_audio(&track), backend.fetch_lyrics(&track));
            let _ = tx.send(BgEvent::AudioReady(track, audio.map_err(|e| e.to_string()), lyrics));
        }));
    }

    /// Enter: open a playlist, or play the selected track.
    pub fn play_selected(&mut self) {
        if self.active_view == ActiveView::Playlists {
            self.load_selected_playlist(false);
        } else if let Some(track) = self.selected_track() {
            self.play_track(track);
        }
    }

    /// Automatic advance (also used to start a freshly queued list).
    pub fn play_next(&mut self) {
        if let Some(next) = self.queue.next_track(self.current_track.clone()) {
            self.play_track(next);
        }
    }

    /// Manual skip ('>'): moves on even when repeating one track.
    pub fn skip_next(&mut self) {
        if let Some(next) = self.queue.skip_track(self.current_track.clone()) {
            self.play_track(next);
        }
    }

    pub fn play_previous(&mut self) {
        if let Some(prev) = self.queue.previous_track(self.current_track.clone()) {
            self.play_track(prev);
        }
    }

    pub fn check_auto_advance(&mut self) {
        if self.audio.is_finished() { self.play_next(); }
    }

    /// 'p': play the selected playlist, or the visible list from the selection on.
    pub fn play_all(&mut self) {
        if self.active_view == ActiveView::Playlists {
            self.load_selected_playlist(true);
            return;
        }
        if self.tracks.is_empty() {
            return;
        }
        self.queue.clear();
        let start = self.selected_index.min(self.tracks.len() - 1);
        for t in self.tracks[start..].iter().chain(self.tracks[..start].iter()) {
            self.queue.enqueue_back(t.clone());
        }
        self.status_message = "Reproduciendo la lista actual...".to_string();
        self.play_next();
    }

    pub fn trigger_prefetch_next(&mut self) {
        let Some(next) = self.queue.peek_next().cloned() else { return };
        let backend = self.backend.clone();
        tokio::spawn(async move { backend.prefetch_audio(&next).await });
    }
}
