use std::sync::Arc;

use calyx_core::{
    CalyxResult, CatalogStore, NativeAudioEngine, PlaybackQueue, PlaybackStatus,
    RepeatMode, SessionConfig, StreamingBackend, Track, UserPlaylist,
};
use ratatui::widgets::ListState;
use crate::artwork::TerminalCoverArt;

/// The streaming service behind the facade; the only place to swap it.
pub type Backend = calyx_core::AppleMusicBackend;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhoneticMode {
    #[default]
    Dual,
    Phonetic,
    Original,
}

impl PhoneticMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Dual => Self::Phonetic,
            Self::Phonetic => Self::Original,
            Self::Original => Self::Dual,
        }
    }
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
    pub language: calyx_core::Language,
    pub phonetic_mode: PhoneticMode,
    pub volume: f32,
    pub should_quit: bool,
    pub(crate) backend: Arc<Backend>,
    pub(crate) store: CatalogStore,
    pub(crate) audio: NativeAudioEngine,
    pub(crate) bg_tx: tokio::sync::mpsc::UnboundedSender<crate::bg_events::BgEvent>,
    pub(crate) bg_rx: tokio::sync::mpsc::UnboundedReceiver<crate::bg_events::BgEvent>,
    pub(crate) play_task: Option<tokio::task::JoinHandle<()>>,
    /// A login window is open; a second one would fight over the CDP port.
    pub(crate) login_in_progress: bool,
}

impl TuiApp {
    pub fn new(storefront: &str) -> CalyxResult<Self> {
        let session = SessionConfig::load_or_default(storefront);
        let language = session.language.unwrap_or_else(calyx_core::Language::detect_system);
        let store = CatalogStore::open_path("calyx_cache.db")?;
        let cached = store.search_tracks("", 50).unwrap_or_default();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut queue_state = ListState::default();
        queue_state.select(Some(0));
        let (bg_tx, bg_rx) = tokio::sync::mpsc::unbounded_channel();

        let audio_cache = calyx_core::AudioCache::new(2 * 1024 * 1024 * 1024);
        let cache_bg = audio_cache.clone();
        tokio::task::spawn_blocking(move || { cache_bg.evict_if_needed(); });
        let backend = Arc::new(Backend::new(session, audio_cache)?);
        let backend_bg = backend.clone();
        tokio::spawn(async move { let _ = backend_bg.ensure_developer_token().await; });

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
            language,
            phonetic_mode: PhoneticMode::default(),
            volume: 0.85,
            should_quit: false,
            backend,
            store,
            audio: NativeAudioEngine::new()?,
            bg_tx,
            bg_rx,
            play_task: None,
            login_in_progress: false,
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
            self.status_message = "Historial de Recientes vaciado por completo ('c').".to_string();
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

    pub fn toggle_phonetic_mode(&mut self) {
        self.phonetic_mode = self.phonetic_mode.next();
        self.status_message = match (self.language, self.phonetic_mode) {
            (calyx_core::Language::Es, PhoneticMode::Dual) => "Modo Fonética: Dual (Original + Romaji/Pinyin)",
            (calyx_core::Language::Es, PhoneticMode::Phonetic) => "Modo Fonética: Solo Fonético (Romaji/Pinyin)",
            (calyx_core::Language::Es, PhoneticMode::Original) => "Modo Fonética: Desactivado (Solo Original)",
            (calyx_core::Language::En, PhoneticMode::Dual) => "Phonetic Mode: Dual (Original + Romaji/Pinyin)",
            (calyx_core::Language::En, PhoneticMode::Phonetic) => "Phonetic Mode: Phonetic Only (Romaji/Pinyin)",
            (calyx_core::Language::En, PhoneticMode::Original) => "Phonetic Mode: Off (Original Only)",
        }.to_string();
    }

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        let _ = self.backend.set_language(self.language);
        self.status_message = match self.language {
            calyx_core::Language::Es => "Idioma: Español ('L' para cambiar)".to_string(),
            calyx_core::Language::En => "Language: English ('L' to switch)".to_string(),
        };
    }

    pub fn is_logged_in(&self) -> bool { self.backend.is_logged_in() }
    pub fn toggle_pause(&mut self) { self.audio.toggle_pause(); }
    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.audio.set_volume(self.volume);
    }
    pub fn playback_status(&self) -> PlaybackStatus { self.audio.status() }
    pub fn current_spectrum(&self) -> calyx_core::SpectrumFrame { self.audio.current_spectrum() }
    pub fn playback_progress(&self) -> (std::time::Duration, std::time::Duration) {
        let (elap, pcm) = self.audio.progress();
        let tot = self.current_track.as_ref().map(|t| std::time::Duration::from_millis(t.duration_ms)).filter(|&d| d > std::time::Duration::ZERO).unwrap_or(pcm);
        (elap, tot)
    }
}
