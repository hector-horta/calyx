//! Remote operations started from the UI. Each one runs on a tokio task
//! against the streaming facade and reports back with a `BgEvent`, so the
//! event loop never waits on the network.

use crate::activity::Job;
use crate::app::{ActiveView, TuiApp};
use crate::bg_events::BgEvent;
use crate::messages::{ListKind, Msg};
use calyx_core::{SectionType, StreamingBackend};

impl TuiApp {
    pub(crate) fn spawn_event<F>(&self, fut: F)
    where
        F: std::future::Future<Output = BgEvent> + Send + 'static,
    {
        let tx = self.bg_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(fut.await);
        });
    }

    pub fn start_search(&mut self) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            return;
        }
        self.say(Msg::Searching(&query));
        self.activity.start(Job::List);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend.search(&query, 25).await.map_err(|e| e.to_string());
            BgEvent::TracksReady {
                view: ActiveView::Home,
                kind: ListKind::Search,
                res,
            }
        });
    }

    pub fn open_section(&mut self, view: ActiveView, section: SectionType, kind: ListKind) {
        self.active_view = view;
        self.say(Msg::Loading(kind));
        self.activity.start(Job::List);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .fetch_section(section)
                .await
                .map_err(|e| e.to_string());
            BgEvent::TracksReady { view, kind, res }
        });
    }

    pub fn sync_library(&mut self) {
        self.active_view = ActiveView::Library;
        self.say(Msg::Loading(ListKind::Library));
        let progress = self.activity.start_counted();
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .fetch_user_library(progress)
                .await
                .map_err(|e| e.to_string());
            BgEvent::TracksReady {
                view: ActiveView::Library,
                kind: ListKind::Library,
                res,
            }
        });
    }

    pub fn sync_playlists(&mut self) {
        self.active_view = ActiveView::Playlists;
        self.fetch_playlists();
    }

    /// Loads the user's playlists without changing the view (also used by 'm').
    pub(crate) fn fetch_playlists(&mut self) {
        self.say(Msg::SyncingPlaylists);
        self.activity.start(Job::Playlists);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            BgEvent::PlaylistsReady(
                backend
                    .fetch_user_playlists()
                    .await
                    .map_err(|e| e.to_string()),
            )
        });
    }

    /// Loads the selected playlist; `play` queues it all instead of browsing it.
    pub fn load_selected_playlist(&mut self, play: bool) {
        let Some(pl) = self
            .playlists
            .get(self.playlist_state.selected().unwrap_or(0))
            .cloned()
        else {
            return;
        };
        self.say(Msg::LoadingPlaylist(&pl.name));
        let progress = self.activity.start_counted();
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .fetch_playlist_tracks(&pl.id, progress)
                .await
                .map_err(|e| e.to_string());
            if play {
                BgEvent::PlaylistReady(pl.name, res)
            } else {
                BgEvent::PlaylistOpened(pl.name, res)
            }
        });
    }

    /// 'o': show the album of the selected track, to play ('p') or enqueue ('A') it.
    pub fn open_selected_album(&mut self) {
        let Some(track) = self.selected_track() else {
            return;
        };
        self.say(Msg::LoadingAlbum(&track.title));
        self.activity.start(Job::List);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            BgEvent::AlbumOpened(
                backend
                    .fetch_album_of(&track)
                    .await
                    .map_err(|e| e.to_string()),
            )
        });
    }

    /// 'f': add the album of the selected track to the user's library.
    pub fn add_selected_album_to_library(&mut self) {
        let Some(track) = self.selected_track() else {
            return;
        };
        self.say(Msg::AddingAlbumToLibrary(&track.album));
        self.activity.start(Job::Write);
        let backend = self.backend.clone();
        self.spawn_event(async move {
            let res = backend
                .add_album_to_library(&track)
                .await
                .map_err(|e| e.to_string());
            BgEvent::AlbumAddedToLibrary(track.album, res)
        });
    }

    pub fn start_login(&mut self) {
        if self.login_in_progress {
            return;
        }
        self.login_in_progress = true;
        let backend = self.backend.clone();
        self.spawn_event(async move {
            BgEvent::LoginDone(backend.login().await.map_err(|e| e.to_string()))
        });
    }
}
