use calyx_core::{
    AppleMusicClient, CalyxResult, CatalogStore, DrmMode, NativeAudioEngine,
    PlaybackQueue, PlaybackStatus, RepeatMode, SessionConfig, Track,
};
use ratatui::widgets::ListState;
use crate::artwork::TerminalCoverArt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Catalog,
    Queue,
}

pub struct TuiApp {
    pub search_input: String,
    pub is_editing: bool,
    pub tracks: Vec<Track>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub queue: PlaybackQueue,
    pub queue_state: ListState,
    pub active_view: ActiveView,
    pub current_track: Option<Track>,
    pub current_artwork: Option<TerminalCoverArt>,
    pub status_message: String,
    #[allow(dead_code)]
    pub drm_mode: DrmMode,
    pub session: SessionConfig,
    pub volume: f32,
    pub should_quit: bool,
    pub(crate) client: AppleMusicClient,
    pub(crate) store: CatalogStore,
    pub(crate) audio: NativeAudioEngine,
}

impl TuiApp {
    pub fn new(storefront: &str) -> CalyxResult<Self> {
        let session = SessionConfig::load_or_default(storefront);
        let mut client = AppleMusicClient::new(&session.storefront)?;
        client.set_tokens(session.developer_token.clone(), session.media_user_token.clone());
        let store = CatalogStore::open_path("calyx_cache.db")?;
        let cached = store.search_tracks("", 50).unwrap_or_default();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut queue_state = ListState::default();
        queue_state.select(Some(0));

        Ok(Self {
            search_input: String::with_capacity(1024),
            is_editing: false,
            tracks: cached,
            selected_index: 0,
            list_state,
            queue: PlaybackQueue::new(),
            queue_state,
            active_view: ActiveView::Catalog,
            current_track: None,
            current_artwork: None,
            status_message: "Listo. '/' Buscar │ 'a' Encolar │ 'n' Siguiente │ 'Tab' Ver Cola │ 'Espacio' Pausa".to_string(),
            drm_mode: DrmMode::WidevineL3Cbcs,
            session,
            volume: 0.85,
            should_quit: false,
            client,
            store,
            audio: NativeAudioEngine::new()?,
        })
    }

    pub fn selected_track(&self) -> Option<Track> {
        match self.active_view {
            ActiveView::Catalog => self.tracks.get(self.selected_index).cloned(),
            ActiveView::Queue => {
                let idx = self.queue_state.selected().unwrap_or(0);
                self.queue.upcoming().get(idx).cloned()
            }
        }
    }

    pub fn next_track(&mut self) {
        match self.active_view {
            ActiveView::Catalog => {
                if !self.tracks.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.tracks.len();
                    self.list_state.select(Some(self.selected_index));
                }
            }
            ActiveView::Queue => {
                let len = self.queue.len();
                if len > 0 {
                    let cur = self.queue_state.selected().unwrap_or(0);
                    let next = (cur + 1) % len;
                    self.queue_state.select(Some(next));
                }
            }
        }
    }

    pub fn previous_track(&mut self) {
        match self.active_view {
            ActiveView::Catalog => {
                if !self.tracks.is_empty() {
                    self.selected_index = if self.selected_index == 0 { self.tracks.len() - 1 } else { self.selected_index - 1 };
                    self.list_state.select(Some(self.selected_index));
                }
            }
            ActiveView::Queue => {
                let len = self.queue.len();
                if len > 0 {
                    let cur = self.queue_state.selected().unwrap_or(0);
                    let prev = if cur == 0 { len - 1 } else { cur - 1 };
                    self.queue_state.select(Some(prev));
                }
            }
        }
    }

    pub fn toggle_view(&mut self) {
        self.active_view = match self.active_view {
            ActiveView::Catalog => ActiveView::Queue,
            ActiveView::Queue => ActiveView::Catalog,
        };
    }

    pub fn enqueue_selected_back(&mut self) {
        if let Some(track) = self.tracks.get(self.selected_index).cloned() {
            self.status_message = format!("Encolada al final: {} — {}", track.artist, track.title);
            self.queue.enqueue_back(track);
        }
    }

    pub fn enqueue_selected_next(&mut self) {
        if let Some(track) = self.tracks.get(self.selected_index).cloned() {
            self.status_message = format!("Encolada a continuación: {} — {}", track.artist, track.title);
            self.queue.enqueue_next(track);
        }
    }

    pub fn remove_selected_from_queue(&mut self) {
        if self.active_view == ActiveView::Queue {
            if let Some(idx) = self.queue_state.selected() {
                if let Some(removed) = self.queue.remove(idx) {
                    self.status_message = format!("Eliminada de la cola: {}", removed.title);
                    if self.queue.is_empty() {
                        self.queue_state.select(None);
                    } else if idx >= self.queue.len() {
                        self.queue_state.select(Some(self.queue.len() - 1));
                    }
                }
            }
        }
    }

    pub fn toggle_repeat(&mut self) -> RepeatMode {
        let mode = self.queue.toggle_repeat();
        self.status_message = format!("Modo Repetición: {}", mode.badge());
        mode
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        let shuf = self.queue.toggle_shuffle();
        self.status_message = format!("Modo Aleatorio (Shuffle): {}", if shuf { "ON 🔀" } else { "OFF" });
        shuf
    }

    pub async fn execute_search(&mut self) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() { return; }
        self.status_message = format!("Buscando '{}'...", query);
        match self.client.search_catalog(&query, 25).await {
            Ok(results) => {
                for track in &results { let _ = self.store.upsert_track(track); }
                self.status_message = format!("{} pistas encontradas.", results.len());
                self.tracks = results;
                self.selected_index = 0;
                self.list_state.select(Some(0));
                self.active_view = ActiveView::Catalog;
            }
            Err(e) => self.status_message = format!("Error de búsqueda: {e}"),
        }
    }

    pub async fn sync_apple_auth(&mut self) {
        self.status_message = "Sincronizando Developer Token...".to_string();
        match self.client.sync_developer_token().await {
            Ok(token) => {
                self.session.developer_token = Some(token.clone());
                let _ = self.session.save();
                let p = &token[..token.len().min(16)];
                self.status_message = format!("Token sincronizado ({p}...)");
            }
            Err(e) => self.status_message = format!("Error en Token: {e}"),
        }
    }

    pub async fn sync_user_auth(&mut self) {
        match self.client.login_and_verify_user().await {
            Ok((dev_tok, user_tok, verified_sf, browser_guard)) => {
                self.session.developer_token = Some(dev_tok);
                self.session.media_user_token = Some(user_tok.clone());
                self.session.storefront = verified_sf.clone();
                let _ = self.session.save();
                if let Some(g) = browser_guard { g.kill_after_save().await; }
                let p = &user_tok[..user_tok.len().min(16)];
                self.status_message = format!("Sesión Apple Music verificada ({p}..., Storefront '{verified_sf}')");
            }
            Err(e) => self.status_message = format!("Error login Apple: {e}"),
        }
    }

    pub async fn toggle_pause(&mut self) { self.audio.toggle_pause(); }
    pub async fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.audio.set_volume(self.volume);
    }
    pub fn playback_status(&self) -> PlaybackStatus { self.audio.status() }
    pub fn playback_progress(&self) -> (std::time::Duration, std::time::Duration) {
        let (elap, pcm) = self.audio.progress();
        let tot = self.current_track.as_ref().map(|t| std::time::Duration::from_millis(t.duration_ms)).filter(|&d| d > std::time::Duration::ZERO).unwrap_or(pcm);
        (elap, tot)
    }
}
