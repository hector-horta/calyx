use calyx_core::Track;
use crate::app::TuiApp;
use crate::artwork::TerminalCoverArt;

impl TuiApp {
    pub async fn play_track(&mut self, mut track: Track) {
        if track.artwork_url.is_none() {
            if let Some(resolved_url) = self.client.resolve_artwork_url(&track).await {
                track.artwork_url = Some(resolved_url);
                let _ = self.store.upsert_track(&track);
            }
        }

        let art_bytes = if let Some(art_url) = track.artwork_url.as_deref() {
            self.client.fetch_stream_bytes(art_url).await.ok()
        } else {
            None
        };

        let seed = format!("{}:{}:{}:{}", track.id, track.artist, track.album, track.title);
        self.current_artwork = Some(TerminalCoverArt::from_jpeg_or_seed(art_bytes.as_deref(), &seed));

        let mut drm_note: Option<String> = None;
        if self.session.has_full_stream_credentials() {
            match self.session.load_wvd_device() {
                Ok(wvd) => {
                    match self.client.fetch_full_track_stream(&track.id, &wvd).await {
                        Ok(full_bytes) => match self.audio.play_encoded_bytes(full_bytes) {
                            Ok(()) => {
                                self.audio.set_volume(self.volume);
                                self.status_message.clear();
                                self.current_track = Some(track);
                                return;
                            }
                            Err(e) => drm_note = Some(format!("Decodificación: {e}")),
                        },
                        Err(e) => drm_note = Some(e.to_string()),
                    }
                }
                Err(_) => {
                    drm_note = Some("Falta device.wvd L3 en carpeta".to_string());
                }
            }
        }

        let Some(url) = track.preview_url.clone() else {
            self.status_message = "La pista no tiene URL de stream disponible.".to_string();
            return;
        };

        match self.client.fetch_stream_bytes(&url).await {
            Ok(bytes) => match self.audio.play_encoded_bytes(bytes) {
                Ok(()) => {
                    self.audio.set_volume(self.volume);
                    let hint = match drm_note {
                        Some(note) => format!("[Preview 30s | {note}]"),
                        None => "[Preview 30s — Presiona 'u' para Login Apple]".to_string(),
                    };
                    self.status_message = hint;
                    self.current_track = Some(track);
                }
                Err(e) => self.status_message = format!("Error de audio: {e}"),
            },
            Err(e) => self.status_message = format!("Error de red en stream: {e}"),
        }
    }

    pub async fn play_selected(&mut self) {
        if let Some(track) = self.selected_track() {
            self.play_track(track).await;
        }
    }

    pub async fn play_next(&mut self) {
        if let Some(next) = self.queue.next_track(self.current_track.clone()) {
            self.play_track(next).await;
        }
    }

    #[allow(dead_code)]
    pub async fn play_previous(&mut self) {
        if let Some(prev) = self.queue.previous_track(self.current_track.clone()) {
            self.play_track(prev).await;
        }
    }

    pub async fn check_auto_advance(&mut self) {
        if self.audio.is_finished() {
            self.play_next().await;
        }
    }
}
