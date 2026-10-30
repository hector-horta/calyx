use calyx_core::{
    AppleMusicClient, CalyxResult, CatalogStore, DrmMode, NativeAudioEngine,
    PlaybackStatus, SessionConfig, Track,
};
use crate::artwork::TerminalCoverArt;

pub struct TuiApp {
    pub search_input: String,
    pub is_editing: bool,
    pub tracks: Vec<Track>,
    pub selected_index: usize,
    pub current_track: Option<Track>,
    pub current_artwork: Option<TerminalCoverArt>,
    pub status_message: String,
    pub drm_mode: DrmMode,
    pub session: SessionConfig,
    pub volume: f32,
    pub should_quit: bool,
    client: AppleMusicClient,
    store: CatalogStore,
    audio: NativeAudioEngine,
}

impl TuiApp {
    pub fn new(storefront: &str) -> CalyxResult<Self> {
        let session = SessionConfig::load_or_default(storefront);
        let mut client = AppleMusicClient::new(&session.storefront)?;
        client.set_tokens(session.developer_token.clone(), session.media_user_token.clone());
        let store = CatalogStore::open_path("calyx_cache.db")?;
        let cached = store.search_tracks("", 50).unwrap_or_default();
        Ok(Self {
            search_input: String::with_capacity(1024),
            is_editing: false,
            tracks: cached,
            selected_index: 0,
            current_track: None,
            current_artwork: None,
            status_message: "Listo. '/' buscar | 'Enter' reproducir | 'a' DevToken | 'u' Login Apple ID | 'Space' pausa | 'q' salir.".to_string(),
            drm_mode: DrmMode::WidevineL3Cbcs,
            session,
            volume: 0.85,
            should_quit: false,
            client,
            store,
            audio: NativeAudioEngine::new()?,
        })
    }

    pub fn next_track(&mut self) {
        if !self.tracks.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.tracks.len();
        }
    }

    pub fn previous_track(&mut self) {
        if !self.tracks.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.tracks.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub async fn execute_search(&mut self) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            return;
        }
        self.status_message = format!("Buscando '{}' en Apple Music...", query);
        match self.client.search_catalog(&query, 25).await {
            Ok(results) => {
                for track in &results {
                    let _ = self.store.upsert_track(track);
                }
                self.status_message = format!("{} pistas encontradas y cacheadas en SQLite WAL.", results.len());
                self.tracks = results;
                self.selected_index = 0;
            }
            Err(e) => self.status_message = format!("Error de búsqueda: {}", e),
        }
    }

    pub async fn play_selected(&mut self) {
        let Some(mut track) = self.tracks.get(self.selected_index).cloned() else {
            return;
        };
        if track.artwork_url.is_none() {
            if let Some(resolved_url) = self.client.resolve_artwork_url(&track).await {
                track.artwork_url = Some(resolved_url);
                if let Some(slot) = self.tracks.get_mut(self.selected_index) {
                    slot.artwork_url = track.artwork_url.clone();
                }
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
                                self.status_message = format!(
                                    "Reproduciendo PISTA COMPLETA ({}) — {} - {} [100% Nativo Rust + .wvd L3]",
                                    track.formatted_duration(),
                                    track.artist,
                                    track.title
                                );
                                self.current_track = Some(track);
                                return;
                            }
                            Err(e) => drm_note = Some(format!("Decodificación: {e}")),
                        },
                        Err(e) => drm_note = Some(e.to_string()),
                    }
                }
                Err(_) => {
                    drm_note = Some("Falta device.wvd L3 en carpeta para desencriptar stream completo".to_string());
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
                        Some(note) => format!("[Preview 30s Nativo | {note}]"),
                        None => "[Preview 30s — Presiona 'u' para Login Apple]".to_string(),
                    };
                    self.status_message = format!("Reproduciendo: {} — {} {hint}", track.artist, track.title);
                    self.current_track = Some(track);
                }
                Err(e) => self.status_message = format!("Error de audio: {}", e),
            },
            Err(e) => self.status_message = format!("Error de red en stream: {}", e),
        }
    }

    pub async fn sync_apple_auth(&mut self) {
        self.status_message = "Extrayendo Developer Token desde music.apple.com...".to_string();
        match self.client.sync_developer_token().await {
            Ok(token) => {
                self.session.developer_token = Some(token.clone());
                let _ = self.session.save();
                let preview = &token[..token.len().min(22)];
                self.status_message = format!("Developer Token sincronizado ({preview}...) y guardado en calyx_auth.json");
            }
            Err(e) => self.status_message = format!("Error extrayendo Developer Token: {e}"),
        }
    }

    pub async fn sync_user_auth(&mut self) {
        match self.client.login_and_verify_user().await {
            Ok((dev_tok, user_tok, verified_sf, browser_guard)) => {
                self.session.developer_token = Some(dev_tok);
                self.session.media_user_token = Some(user_tok.clone());
                self.session.storefront = verified_sf.clone();
                let save_res = self.session.save();
                let reused = browser_guard.is_none();
                if let Some(g) = browser_guard { g.kill_after_save().await; }
                let preview = &user_tok[..user_tok.len().min(18)];
                if let Err(e) = save_res {
                    self.status_message = format!("Sesión verificada pero falló al guardar calyx_auth.json: {e}");
                } else {
                    let how = if reused { "ya estaba activa (sin abrir navegador)" } else { "guardada y ventana cerrada" };
                    self.status_message = format!("Sesión Apple Music {how} ({preview}..., Storefront '{verified_sf}')");
                }
            }
            Err(e) => self.status_message = format!("Login Apple Music cancelado o con error: {e}"),
        }
    }

    pub async fn toggle_pause(&mut self) { self.audio.toggle_pause(); }

    pub async fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.audio.set_volume(self.volume);
    }

    pub fn playback_status(&self) -> PlaybackStatus { self.audio.status() }

    pub fn playback_progress(&self) -> (std::time::Duration, std::time::Duration) {
        self.audio.progress()
    }
}
