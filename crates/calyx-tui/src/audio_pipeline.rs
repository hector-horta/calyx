//! Moves M4A decoding + spectrum precompute off the UI thread.

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use calyx_core::{DecodedTrack, LyricLine, Track, decode_track};

impl TuiApp {
    /// Decodes `bytes` on tokio's blocking pool and reports back via `BgEvent::AudioDecoded`.
    pub(crate) fn spawn_decode(
        &self,
        track: Track,
        bytes: Vec<u8>,
        lyrics: Vec<LyricLine>,
        hint: String,
    ) {
        let tx = self.bg_tx.clone();
        tokio::task::spawn_blocking(move || {
            let res = decode_track(bytes).map_err(|e| e.to_string());
            let _ = tx.send(BgEvent::AudioDecoded(track, res, lyrics, hint));
        });
    }

    /// Starts playback of a decoded track only if it is still the current track
    /// (a stale decode finishing after the user skipped is dropped).
    pub(crate) fn on_audio_decoded(
        &mut self,
        track: Track,
        res: Result<DecodedTrack, String>,
        lyrics: Vec<LyricLine>,
        hint: String,
    ) {
        if self
            .current_track
            .as_ref()
            .map(|t| t.id != track.id)
            .unwrap_or(true)
        {
            return;
        }

        self.activity.finish(crate::activity::Job::Audio);
        self.current_lyrics = lyrics;
        match res {
            Ok(decoded) => match self.audio.play_decoded(decoded) {
                Ok(()) => {
                    self.audio.set_volume(self.volume);
                    self.status_message = hint;
                    self.current_track = Some(track);
                    self.trigger_prefetch_next();
                }
                Err(e) => self.say(crate::messages::Msg::AudioError(&e.to_string())),
            },
            Err(e) => self.say(crate::messages::Msg::AudioError(&e.to_string())),
        }
    }
}
