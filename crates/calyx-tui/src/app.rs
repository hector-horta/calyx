use calyx_core::{
    AppleMusicClient, CalyxResult, CatalogStore, DrmMode, NativeAudioEngine,
    PlaybackQueue, PlaybackStatus, RepeatMode, SessionConfig, Track, UserPlaylist,
};
use ratatui::widgets::ListState;
use crate::artwork::TerminalCoverArt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Home,
    New,
    Radio,
    Library,
    Playlists,
    Queue,
    Help,
}

pub struct TuiApp {
    pub search_input: String,
    pub is_editing: bool,
    pub tracks: Vec<Track>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub queue: PlaybackQueue,
    pub queue_state: ListState,
    pub playlists: Vec<UserPlaylist>,
    pub playlist_state: ListState,
    pub active_view: ActiveView,
    pub current_track: Option<Track>,
    pub current_artwork: Option<TerminalCoverArt>,
    pub play_icon: crate::artwork::TerminalIcon,
    pub pause_icon: crate::artwork::TerminalIcon,
    pub current_lyrics: Vec<calyx_core::LyricLine>,
    pub status_message: String,
    #[allow(dead_code)]
    pub drm_mode: DrmMode,
    pub language: calyx_core::Language,
    pub session: SessionConfig,
    pub volume: f32,
    pub should_quit: bool,
    pub(crate) client: AppleMusicClient,
    pub(crate) store: CatalogStore,
    pub(crate) audio: NativeAudioEngine,
    pub(crate) bg_tx: tokio::sync::mpsc::UnboundedSender<crate::playback::BgEvent>,
    pub(crate) bg_rx: tokio::sync::mpsc::UnboundedReceiver<crate::playback::BgEvent>,
    pub(crate) play_task: Option<tokio::task::JoinHandle<()>>,
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
        let (bg_tx, bg_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            search_input: String::with_capacity(1024),
            is_editing: false,
            tracks: cached,
            selected_index: 0,
            playlists: Vec::new(),
            playlist_state: ListState::default(),
            list_state,
            queue: PlaybackQueue::new(),
            queue_state,
            active_view: ActiveView::Home,
            current_track: None,
            current_artwork: None,
            play_icon: crate::artwork::TerminalIcon::load_or_fallback(
                &["assets/icons/transparent-play.png", "assets/icons/play.png"],
                6, 6,
                [" █◣   ", " ██◣  ", " █◤   "],
                ratatui::style::Color::Cyan,
            ),
            pause_icon: crate::artwork::TerminalIcon::load_or_fallback(
                &["assets/icons/transparent - pause.png", "assets/icons/pause.png"],
                6, 6,
                [" █ █  ", " █ █  ", " █ █  "],
                ratatui::style::Color::Yellow,
            ),
            current_lyrics: Vec::new(),
            status_message: String::new(),
            drm_mode: DrmMode::WidevineL3Cbcs,
            language: session.language.unwrap_or_else(calyx_core::Language::detect_system),
            session,
            volume: 0.85,
            should_quit: false,
            client,
            store,
            audio: NativeAudioEngine::new()?,
            bg_tx,
            bg_rx,
            play_task: None,
        })
    }

pub fn selected_track(&self) -> Option<Track> {
    match self.active_view {
        ActiveView::Help | ActiveView::Playlists => None,
        ActiveView::Queue => self.queue.upcoming().get(self.queue_state.selected().unwrap_or(0)).cloned(),
        _ => self.tracks.get(self.selected_index).cloned(),
    }
}

    pub fn next_track(&mut self) {
        match self.active_view {
            ActiveView::Help => {}
            ActiveView::Playlists => if !self.playlists.is_empty() {
                let next = (self.playlist_state.selected().unwrap_or(0) + 1) % self.playlists.len();
                self.playlist_state.select(Some(next));
            },
            ActiveView::Queue => if !self.queue.is_empty() {
                let next = (self.queue_state.selected().unwrap_or(0) + 1) % self.queue.len();
                self.queue_state.select(Some(next));
            },
            _ => if !self.tracks.is_empty() {
                self.selected_index = (self.selected_index + 1) % self.tracks.len();
                self.list_state.select(Some(self.selected_index));
            },
        }
    }

    pub fn previous_track(&mut self) {
        match self.active_view {
            ActiveView::Help => {}
            ActiveView::Playlists => if !self.playlists.is_empty() {
                let cur = self.playlist_state.selected().unwrap_or(0);
                self.playlist_state.select(Some(if cur == 0 { self.playlists.len() - 1 } else { cur - 1 }));
            },
            ActiveView::Queue => if !self.queue.is_empty() {
                let cur = self.queue_state.selected().unwrap_or(0);
                self.queue_state.select(Some(if cur == 0 { self.queue.len() - 1 } else { cur - 1 }));
            },
            _ => if !self.tracks.is_empty() {
                self.selected_index = if self.selected_index == 0 { self.tracks.len() - 1 } else { self.selected_index - 1 };
                self.list_state.select(Some(self.selected_index));
            },
        }
    }

    pub fn toggle_view(&mut self) {
        self.active_view = if self.active_view == ActiveView::Queue { ActiveView::Home } else { ActiveView::Queue };
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
        } else if self.active_view == ActiveView::Home && !self.tracks.is_empty() {
            let removed = self.tracks.remove(self.selected_index);
            let _ = self.store.delete_track(&removed.id);
            if self.tracks.is_empty() {
                self.selected_index = 0;
                self.list_state.select(None);
            } else if self.selected_index >= self.tracks.len() {
                self.selected_index = self.tracks.len() - 1;
                self.list_state.select(Some(self.selected_index));
            }
            self.status_message = format!("Eliminada de Recientes: {}", removed.title);
        }
    }

    pub fn clear_current_list(&mut self) {
        if self.active_view == ActiveView::Queue {
            self.queue.clear();
            self.queue_state.select(None);
            self.status_message = "Cola de reproducción vaciada ('c').".to_string();
        } else if self.active_view == ActiveView::Home {
            let _ = self.store.clear_all_tracks();
            self.tracks.clear();
            self.selected_index = 0;
            self.list_state.select(None);
            self.status_message = "Historial de Recientes (SQLite) vaciado por completo ('c').".to_string();
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

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        self.session.language = Some(self.language);
        let _ = self.session.save();
        self.status_message = match self.language {
            calyx_core::Language::Es => "Idioma: Español ('L' para cambiar)".to_string(),
            calyx_core::Language::En => "Language: English ('L' to switch)".to_string(),
        };
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
                self.active_view = ActiveView::Home;
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
    pub fn current_spectrum(&self) -> [u8; 32] { self.audio.current_spectrum() }
    pub fn playback_progress(&self) -> (std::time::Duration, std::time::Duration) {
        let (elap, pcm) = self.audio.progress();
        let tot = self.current_track.as_ref().map(|t| std::time::Duration::from_millis(t.duration_ms)).filter(|&d| d > std::time::Duration::ZERO).unwrap_or(pcm);
        (elap, tot)
    }
}
