//! The OS media controls: Windows SMTC, Linux MPRIS (D-Bus) and macOS Now
//! Playing, through `souvlaki`. Apps such as ListeningTo read what calyx
//! plays from there, and the media keys come back as events. Optional: with
//! no media service (SSH without a session bus) it is simply off.

use calyx_core::now_playing::{NowPlaying, PlayState};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

pub struct OsMedia {
    controls: MediaControls,
    events: Receiver<MediaControlEvent>,
    /// Kept alive (and pumped) for as long as the controls exist.
    #[cfg(windows)]
    window: crate::os_media_window::HiddenWindow,
}

impl OsMedia {
    /// Registers calyx with the OS media controls; `None` when the OS has
    /// none to offer (it is logged at debug level, never shown as an error).
    pub fn start() -> Option<Self> {
        #[cfg(windows)]
        let window = crate::os_media_window::HiddenWindow::create()?;
        #[cfg(windows)]
        let hwnd = Some(window.handle());
        #[cfg(not(windows))]
        let hwnd = None;

        let config = PlatformConfig {
            display_name: "calyx",
            dbus_name: "calyx",
            hwnd,
        };

        let mut controls = match MediaControls::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(error = ?e, "no OS media controls");
                return None;
            }
        };

        let (tx, events) = channel();
        // A closed receiver (calyx exiting) just drops the key press.
        let forward = move |event| drop(tx.send(event));
        if let Err(e) = controls.attach(forward) {
            tracing::debug!(error = ?e, "media keys unavailable");
            return None;
        }

        Some(Self {
            controls,
            events,
            #[cfg(windows)]
            window,
        })
    }

    /// Updates the OS media controls with the current playback state and metadata.
    pub fn show(&mut self, np: &NowPlaying, new_song: bool) {
        if new_song {
            let metadata = MediaMetadata {
                title: Some(&np.title),
                artist: Some(&np.artist),
                album: Some(&np.album),
                cover_url: np.artwork_url.as_deref(),
                duration: Some(Duration::from_millis(np.duration_ms)),
            };
            if let Err(e) = self.controls.set_metadata(metadata) {
                tracing::debug!(error = ?e, "failed to set metadata");
            }
        }

        let progress = Some(MediaPosition(Duration::from_millis(np.position_ms)));
        let playback = match np.state {
            PlayState::Playing => MediaPlayback::Playing { progress },
            PlayState::Paused => MediaPlayback::Paused { progress },
        };
        if let Err(e) = self.controls.set_playback(playback) {
            tracing::debug!(error = ?e, "failed to set playback state");
        }
    }

    /// Nothing is playing any more.
    pub fn show_stopped(&mut self) {
        if let Err(e) = self.controls.set_playback(MediaPlayback::Stopped) {
            tracing::debug!(error = ?e, "failed to set playback state");
        }
    }

    /// The next media key pressed, if any. Pumps the hidden window first
    /// on Windows, where SMTC talks to it through window messages.
    pub fn next_event(&self) -> Option<MediaControlEvent> {
        #[cfg(windows)]
        self.window.pump();
        self.events.try_recv().ok()
    }
}
