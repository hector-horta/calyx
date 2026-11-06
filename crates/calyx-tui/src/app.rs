use std::sync::Arc;

use crate::artwork::TerminalCoverArt;
use crate::messages::Msg;
use calyx_core::{
    CalyxResult, CatalogStore, NativeAudioEngine, PlaybackQueue, PlaybackStatus, RepeatMode,
    SessionConfig, StreamingBackend, Track, UserPlaylist,
};
use ratatui::widgets::ListState;

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
    /// Next track being prefetched into the audio cache (spinner in the lists).
    pub(crate) caching: crate::spinner::Caching,
    /// Open 'm' picker (add the selected song to a playlist).
    pub(crate) picker: Option<crate::picker::PlaylistPicker>,
    /// Background work in progress (spinner and counter in the status line).
    pub(crate) activity: crate::activity::Activity,
}

impl TuiApp {
    pub fn new(storefront: &str) -> CalyxResult<Self> {
        let session = SessionConfig::load_or_default(storefront);
        let language = session
            .language
            .unwrap_or_else(calyx_core::Language::detect_system);
        let db_path = calyx_core::paths::catalog_db();
        // Versions before 0.1.7 kept the history next to where calyx was launched.
        // The WAL is only copied together with its own database file.
        if calyx_core::paths::adopt_legacy_file(std::path::Path::new("calyx_cache.db"), &db_path) {
            let wal = std::path::PathBuf::from(format!("{}-wal", db_path.display()));
            calyx_core::paths::adopt_legacy_file(std::path::Path::new("calyx_cache.db-wal"), &wal);
        }
        let store = CatalogStore::open_path(&db_path)?;
        let cached = store.search_tracks("", 50).unwrap_or_default();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut queue_state = ListState::default();
        queue_state.select(Some(0));
        let (bg_tx, bg_rx) = tokio::sync::mpsc::unbounded_channel();

        let audio_cache = calyx_core::AudioCache::new(2 * 1024 * 1024 * 1024);
        let cache_bg = audio_cache.clone();
        tokio::task::spawn_blocking(move || {
            cache_bg.evict_if_needed();
        });
        let backend = Arc::new(Backend::new(session, audio_cache)?);
        let backend_bg = backend.clone();
        tokio::spawn(async move {
            let _ = backend_bg.ensure_developer_token().await;
        });

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
                6,
                6,
                [" █◣   ", " ██◣  ", " █◤   "],
                ratatui::style::Color::Cyan,
            ),
            pause_icon: crate::artwork::TerminalIcon::load_or_fallback(
                &[
                    "assets/icons/transparent - pause.png",
                    "assets/icons/pause.png",
                ],
                6,
                6,
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
            caching: None,
            picker: None,
            activity: Default::default(),
        })
    }

    pub fn selected_track(&self) -> Option<Track> {
        match self.active_view {
            ActiveView::Help | ActiveView::Playlists => None,
            ActiveView::Queue => self
                .queue
                .upcoming()
                .get(self.queue_state.selected().unwrap_or(0))
                .cloned(),
            _ => self.tracks.get(self.selected_index).cloned(),
        }
    }

    pub fn toggle_repeat(&mut self) -> RepeatMode {
        let mode = self.queue.toggle_repeat();
        self.say(Msg::Repeat(mode));
        mode
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        let shuf = self.queue.toggle_shuffle();
        self.say(Msg::Shuffle(shuf));
        shuf
    }

    pub fn toggle_phonetic_mode(&mut self) {
        self.phonetic_mode = self.phonetic_mode.next();
        self.status_message = match (self.language, self.phonetic_mode) {
            (calyx_core::Language::Es, PhoneticMode::Dual) => {
                "Modo Fonética: Dual (Original + Romaji/Pinyin)"
            }
            (calyx_core::Language::Es, PhoneticMode::Phonetic) => {
                "Modo Fonética: Solo Fonético (Romaji/Pinyin)"
            }
            (calyx_core::Language::Es, PhoneticMode::Original) => {
                "Modo Fonética: Desactivado (Solo Original)"
            }
            (calyx_core::Language::En, PhoneticMode::Dual) => {
                "Phonetic Mode: Dual (Original + Romaji/Pinyin)"
            }
            (calyx_core::Language::En, PhoneticMode::Phonetic) => {
                "Phonetic Mode: Phonetic Only (Romaji/Pinyin)"
            }
            (calyx_core::Language::En, PhoneticMode::Original) => {
                "Phonetic Mode: Off (Original Only)"
            }
        }
        .to_string();
    }

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        let _ = self.backend.set_language(self.language);
        self.say(Msg::LanguageSet);
    }

    /// Shows `msg` in the status line, in the current language.
    pub fn say(&mut self, msg: Msg<'_>) {
        self.status_message = msg.text(self.language);
    }

    pub fn is_logged_in(&self) -> bool {
        self.backend.is_logged_in()
    }
    pub fn toggle_pause(&mut self) {
        self.audio.toggle_pause();
    }
    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.audio.set_volume(self.volume);
    }
    pub fn playback_status(&self) -> PlaybackStatus {
        self.audio.status()
    }
    pub fn current_spectrum(&self) -> calyx_core::SpectrumFrame {
        self.audio.current_spectrum()
    }
    pub fn playback_progress(&self) -> (std::time::Duration, std::time::Duration) {
        let (elap, pcm) = self.audio.progress();
        let tot = self
            .current_track
            .as_ref()
            .map(|t| std::time::Duration::from_millis(t.duration_ms))
            .filter(|&d| d > std::time::Duration::ZERO)
            .unwrap_or(pcm);
        (elap, tot)
    }
}
