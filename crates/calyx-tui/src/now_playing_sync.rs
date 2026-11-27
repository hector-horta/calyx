//! Keeps the OS media controls and `now_playing.json` in step with the
//! player (what ListeningTo reads), and turns media keys into player actions.

use crate::app::TuiApp;
use calyx_core::PlaybackStatus;
use calyx_core::now_playing::{self, NowPlaying, PlayState, REFRESH_EVERY, SCHEMA};
use souvlaki::MediaControlEvent;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// How often the OS gets the position while playing (MPRIS readers poll it).
const POSITION_EVERY: Duration = Duration::from_secs(1);

/// What was last published: (track id, play state).
type Shown = (String, PlayState);

/// What a sync has to do.
#[derive(Debug, Default, PartialEq)]
struct Due {
    new_song: bool,
    os: bool,
    file: bool,
    clear: bool,
}

/// Determines what needs to be updated in the now-playing state.
fn due(
    shown: Option<&Shown>,
    next: Option<&Shown>,
    os_at: Option<Instant>,
    file_at: Option<Instant>,
) -> Due {
    match next {
        None => Due {
            clear: shown.is_some(),
            ..Due::default()
        },
        Some(k) => {
            let changed = shown != Some(k);
            let new_song = shown.is_none_or(|s| s.0 != k.0);
            let os = changed
                || (k.1 == PlayState::Playing
                    && os_at.is_none_or(|t| t.elapsed() >= POSITION_EVERY));
            let file = changed || file_at.is_none_or(|t| t.elapsed() >= REFRESH_EVERY);
            Due {
                new_song,
                os,
                file,
                clear: false,
            }
        }
    }
}

/// Off by default (`--no-now-playing`); `start` turns it on.
#[derive(Default)]
pub struct Publisher {
    os: Option<crate::os_media::OsMedia>,
    file: Option<PathBuf>,
    shown: Option<Shown>,
    os_at: Option<Instant>,
    file_at: Option<Instant>,
}

impl Publisher {
    /// On: the OS media controls when the OS has them, and the JSON file.
    pub fn start() -> Self {
        Self {
            os: crate::os_media::OsMedia::start(),
            file: Some(now_playing::now_playing_file()),
            shown: None,
            os_at: None,
            file_at: None,
        }
    }

    /// The next media key pressed, if the OS controls are on.
    pub fn next_event(&self) -> Option<MediaControlEvent> {
        self.os.as_ref()?.next_event()
    }

    /// Publishes the new track state to OS controls and JSON file based on synchronization requirements.
    pub fn publish(&mut self, next: Option<(String, NowPlaying)>) {
        let key = next.as_ref().map(|(id, np)| (id.clone(), np.state));
        let due = due(self.shown.as_ref(), key.as_ref(), self.os_at, self.file_at);

        if due.clear {
            if let Some(os) = self.os.as_mut() {
                os.show_stopped();
            }
            if let Some(path) = &self.file {
                now_playing::clear(path);
            }
            self.shown = None;
            return;
        }

        let Some((_, np)) = next else {
            return;
        };

        if due.os {
            if let Some(os) = self.os.as_mut() {
                os.show(&np, due.new_song);
            }
            self.os_at = Some(Instant::now());
        }

        if due.file {
            if let Some(path) = &self.file
                && let Err(e) = np.write_to(path)
            {
                tracing::debug!(error = %e, "could not write now_playing.json");
            }
            self.file_at = Some(Instant::now());
        }

        self.shown = key;
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        if let (Some(path), Some(_)) = (&self.file, &self.shown) {
            now_playing::clear(path);
        }
    }
}

impl TuiApp {
    /// What to publish: outer `None` keeps the last state (the next song is
    /// loading, so the OS does not flicker to "stopped" between songs);
    /// `Some(None)` means nothing is playing; otherwise (track id, snapshot).
    fn now_playing_snapshot(&self) -> Option<Option<(String, NowPlaying)>> {
        let state = match self.playback_status() {
            PlaybackStatus::Playing => PlayState::Playing,
            PlaybackStatus::Paused => PlayState::Paused,
            _ => {
                return if self.activity.busy(&[crate::activity::Job::Audio]) {
                    None
                } else {
                    Some(None)
                };
            }
        };

        let Some(track) = self.current_track.as_ref() else {
            return Some(None);
        };
        let (elapsed, total) = self.playback_progress();
        let duration_ms = total.as_millis() as u64;
        let position_ms = (elapsed.as_millis() as u64).min(duration_ms);

        Some(Some((
            track.id.clone(),
            NowPlaying {
                schema: SCHEMA,
                player: "calyx",
                state,
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                artwork_url: track.artwork_url.clone(),
                duration_ms,
                position_ms,
                updated_at_ms: now_playing::now_ms(),
            },
        )))
    }

    /// Publishes the current song (called on every turn of the event loop;
    /// the publisher only does work when something is due).
    pub fn sync_now_playing(&mut self) {
        if let Some(next) = self.now_playing_snapshot() {
            self.now_playing.publish(next);
        }
    }

    /// Applies the media keys pressed since the last turn (play/pause, next,
    /// previous, stop). Seeking is not supported and is ignored.
    pub fn handle_media_keys(&mut self) {
        while let Some(event) = self.now_playing.next_event() {
            let status = self.playback_status();
            match event {
                MediaControlEvent::Play if status == PlaybackStatus::Paused => self.toggle_pause(),
                MediaControlEvent::Pause if status == PlaybackStatus::Playing => {
                    self.toggle_pause()
                }
                MediaControlEvent::Toggle => self.toggle_pause(),
                MediaControlEvent::Next => self.skip_next(),
                MediaControlEvent::Previous => self.play_previous(),
                MediaControlEvent::Stop => self.audio.stop(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "now_playing_sync_tests.rs"]
mod tests;
