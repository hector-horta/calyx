//! Results of background work (network, decoding) delivered to the UI thread.
//! Handlers only touch app state; nothing here awaits.

use crate::app::{ActiveView, TuiApp};
use crate::artwork::TerminalCoverArt;
use crate::messages::{ListKind, Msg};
use calyx_core::{AudioSource, DecodedTrack, LyricLine, ResolvedAudio, Track, UserPlaylist};

pub enum BgEvent {
    AudioReady(Track, Result<ResolvedAudio, String>, Vec<LyricLine>),
    AudioDecoded(Track, Result<DecodedTrack, String>, Vec<LyricLine>, String),
    ArtworkReady(String, TerminalCoverArt),
    /// Search, curated sections and the user library.
    TracksReady {
        view: ActiveView,
        kind: ListKind,
        res: Result<Vec<Track>, String>,
    },
    PlaylistsReady(Result<Vec<UserPlaylist>, String>),
    /// A playlist opened with Enter (browse its tracks).
    PlaylistOpened(String, Result<Vec<Track>, String>),
    /// A playlist started with 'p' (queue all and play).
    PlaylistReady(String, Result<Vec<Track>, String>),
    LoginDone(Result<(), String>),
    /// Album opened with 'o': its name and songs.
    AlbumOpened(Result<(String, Vec<Track>), String>),
    /// The prefetch of this track id finished (cached, or failed).
    PrefetchDone(String),
}

/// Quality hint shown in the status line for a resolved buffer.
fn audio_hint(audio: &ResolvedAudio, lang: calyx_core::Language) -> String {
    match audio.source {
        AudioSource::Cache => Msg::HintCache,
        AudioSource::FullStream => Msg::HintFull,
        AudioSource::Preview => Msg::HintPreview(audio.note.as_deref()),
    }
    .text(lang)
}

impl TuiApp {
    pub fn poll_bg_events(&mut self) {
        while let Ok(ev) = self.bg_rx.try_recv() {
            self.handle_bg_event(ev);
        }
    }

    fn is_current(&self, track_id: &str) -> bool {
        self.current_track
            .as_ref()
            .map(|t| t.id == track_id)
            .unwrap_or(true)
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
                    let hint = audio_hint(&audio, self.language);
                    self.spawn_decode(track, audio.bytes, lyrics, hint);
                }
                Err(e) => {
                    self.current_lyrics = lyrics;
                    self.status_message = e;
                }
            },
            BgEvent::AudioReady(..) => {}
            BgEvent::AudioDecoded(track, res, lyrics, hint) => {
                self.on_audio_decoded(track, res, lyrics, hint)
            }
            BgEvent::ArtworkReady(tid, art) if self.is_current(&tid) => {
                self.current_artwork = Some(art)
            }
            BgEvent::ArtworkReady(..) => {}
            BgEvent::TracksReady {
                view,
                kind,
                res: Ok(list),
            } => {
                // An empty library sync keeps whatever is on screen.
                if view == ActiveView::Library && list.is_empty() {
                    self.say(Msg::ListEmpty(kind));
                    return;
                }
                if view == ActiveView::Home {
                    for t in &list {
                        let _ = self.store.upsert_track(t);
                    }
                }
                self.say(Msg::ListLoaded {
                    kind,
                    count: list.len(),
                });
                self.show_tracks(list, view);
            }
            BgEvent::TracksReady {
                kind, res: Err(e), ..
            } => self.say(Msg::ListError { kind, err: &e }),
            BgEvent::PlaylistsReady(Ok(playlists)) => {
                self.say(Msg::PlaylistsSynced(playlists.len()));
                self.playlist_state
                    .select(if playlists.is_empty() { None } else { Some(0) });
                self.playlists = playlists;
            }
            BgEvent::PlaylistsReady(Err(e)) => self.say(Msg::PlaylistsError(&e)),
            BgEvent::PlaylistOpened(name, Ok(list)) => {
                self.say(Msg::PlaylistOpened {
                    name: &name,
                    count: list.len(),
                });
                self.show_tracks(list, ActiveView::Library);
            }
            BgEvent::PlaylistOpened(name, Err(e)) => self.say(Msg::PlaylistError {
                name: &name,
                err: &e,
            }),
            BgEvent::PlaylistReady(name, Ok(list)) if !list.is_empty() => {
                self.queue.clear();
                for t in &list {
                    self.queue.enqueue_back(t.clone());
                }
                self.show_tracks(list, ActiveView::Library);
                self.say(Msg::PlayingPlaylist {
                    name: &name,
                    count: self.queue.len(),
                });
                self.play_next();
            }
            BgEvent::PlaylistReady(name, _) => self.say(Msg::PlaylistPlayFailed(&name)),
            BgEvent::LoginDone(res) => {
                self.login_in_progress = false;
                match res {
                    Ok(()) => self.say(Msg::LoginOk),
                    Err(e) => self.say(Msg::LoginError(&e)),
                }
            }
            // The album replaces the visible list; from the queue it opens in Home.
            BgEvent::AlbumOpened(Ok((name, list))) if !list.is_empty() => {
                self.say(Msg::AlbumOpened {
                    name: &name,
                    count: list.len(),
                });
                let view = match self.active_view {
                    ActiveView::Queue | ActiveView::Help | ActiveView::Playlists => {
                        ActiveView::Home
                    }
                    v => v,
                };
                self.show_tracks(list, view);
            }
            BgEvent::AlbumOpened(Ok((name, _))) => self.say(Msg::AlbumEmpty(&name)),
            BgEvent::AlbumOpened(Err(e)) => self.say(Msg::AlbumError(&e)),
            // A newer prefetch may already own the spinner; only clear our own.
            BgEvent::PrefetchDone(id) => {
                if self.caching.as_ref().is_some_and(|(c, _)| *c == id) {
                    self.caching = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audio(source: AudioSource, note: Option<&str>) -> ResolvedAudio {
        ResolvedAudio {
            bytes: Vec::new(),
            source,
            note: note.map(str::to_string),
        }
    }

    #[test]
    fn hint_reflects_where_audio_came_from() {
        use calyx_core::Language::{En, Es};
        assert_eq!(
            audio_hint(&audio(AudioSource::Cache, None), Es),
            "[AAC 256k | Caché]"
        );
        assert_eq!(
            audio_hint(&audio(AudioSource::Cache, None), En),
            "[AAC 256k | Cached]"
        );
        assert_eq!(
            audio_hint(&audio(AudioSource::FullStream, None), Es),
            "[AAC 256k]"
        );
        assert_eq!(
            audio_hint(&audio(AudioSource::Preview, Some("HTTP 403")), Es),
            "[Preview 30s | HTTP 403]"
        );
        assert!(audio_hint(&audio(AudioSource::Preview, None), En).contains("log in"));
    }
}
