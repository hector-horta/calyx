use crate::decode::{DecodedTrack, decode_track};
use crate::error::{CalyxError, CalyxResult};
use crate::models::PlaybackStatus;
use crate::spectrum::{FRAME_MS, SPECTRUM_BANDS, SpectrumFrame};
use rodio::{OutputStream, OutputStreamHandle, Sink, buffer::SamplesBuffer};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tracing::info;

/// Everything that changes during playback, behind one lock so readers never
/// see a half-updated state and there is no lock ordering to get wrong.
struct PlayerState {
    sink: Option<Sink>,
    volume: f32,
    duration: Duration,
    /// When the current playing stretch started (None while paused/stopped).
    started_at: Option<Instant>,
    /// Time played before the current stretch (sum of earlier stretches).
    accumulated: Duration,
    status: PlaybackStatus,
    spectrum: Vec<SpectrumFrame>,
}

impl PlayerState {
    fn elapsed(&self) -> Duration {
        self.accumulated + self.started_at.map(|t| t.elapsed()).unwrap_or_default()
    }
}

pub struct NativeAudioEngine {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    state: Mutex<PlayerState>,
}

impl NativeAudioEngine {
    pub fn new() -> CalyxResult<Self> {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                info!("Audio output device unavailable ({e}), running in headless mode");
                (None, None)
            }
        };
        Ok(Self::with_output(stream, handle))
    }

    fn with_output(stream: Option<OutputStream>, handle: Option<OutputStreamHandle>) -> Self {
        Self {
            _stream: stream,
            handle,
            state: Mutex::new(PlayerState {
                sink: None,
                volume: 0.85,
                duration: Duration::ZERO,
                started_at: None,
                accumulated: Duration::ZERO,
                status: PlaybackStatus::Stopped,
                spectrum: Vec::new(),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, PlayerState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn play_encoded_bytes(&self, bytes: Vec<u8>) -> CalyxResult<()> {
        self.play_decoded(decode_track(bytes)?)
    }

    pub fn play_decoded(&self, track: DecodedTrack) -> CalyxResult<()> {
        let DecodedTrack {
            channels,
            sample_rate,
            pcm,
            spectrum,
        } = track;
        let duration =
            Duration::from_secs(pcm.len() as u64 / (channels as u64 * sample_rate as u64).max(1));
        let volume = self.lock().volume;
        // Build the sink without holding the lock.
        let new_sink = match &self.handle {
            Some(handle) => {
                let sink = Sink::try_new(handle)
                    .map_err(|e| CalyxError::Audio(format!("Failed to create audio sink: {e}")))?;
                sink.set_volume(volume);
                sink.append(SamplesBuffer::new(channels, sample_rate, pcm));
                sink.play();
                Some(sink)
            }
            None => None,
        };
        let mut st = self.lock();
        if let Some(old) = st.sink.take() {
            old.stop();
        }
        st.sink = new_sink;
        st.duration = duration;
        st.spectrum = spectrum;
        st.accumulated = Duration::ZERO;
        st.started_at = Some(Instant::now());
        st.status = PlaybackStatus::Playing;
        Ok(())
    }

    pub fn toggle_pause(&self) {
        let mut st = self.lock();
        match st.status {
            PlaybackStatus::Playing => {
                if let Some(sink) = st.sink.as_ref() {
                    sink.pause();
                }
                st.accumulated = st.elapsed();
                st.started_at = None;
                st.status = PlaybackStatus::Paused;
            }
            PlaybackStatus::Paused => {
                if let Some(sink) = st.sink.as_ref() {
                    sink.play();
                }
                st.started_at = Some(Instant::now());
                st.status = PlaybackStatus::Playing;
            }
            _ => {}
        }
    }

    pub fn stop(&self) {
        let mut st = self.lock();
        if let Some(sink) = st.sink.take() {
            sink.stop();
        }
        st.started_at = None;
        st.accumulated = Duration::ZERO;
        st.status = PlaybackStatus::Stopped;
    }

    pub fn set_volume(&self, volume: f32) {
        let mut st = self.lock();
        st.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = st.sink.as_ref() {
            sink.set_volume(st.volume);
        }
    }

    /// (elapsed, total) of the current track.
    pub fn progress(&self) -> (Duration, Duration) {
        let st = self.lock();
        (st.elapsed(), st.duration)
    }

    /// True once, when a playing track runs out; the engine then stops.
    pub fn is_finished(&self) -> bool {
        let mut st = self.lock();
        // Without an audio device there is no sink to drain: go by the clock.
        let ran_out = match st.sink.as_ref() {
            Some(sink) => sink.empty(),
            None => st.elapsed() >= st.duration,
        };
        let finished = st.status == PlaybackStatus::Playing && ran_out;
        if finished {
            st.started_at = None;
            st.status = PlaybackStatus::Stopped;
        }
        finished
    }

    pub fn status(&self) -> PlaybackStatus {
        self.lock().status
    }

    pub fn current_spectrum(&self) -> SpectrumFrame {
        const SILENT: SpectrumFrame = [0u8; SPECTRUM_BANDS];
        let st = self.lock();
        if st.status == PlaybackStatus::Stopped {
            return SILENT;
        }
        let idx = (st.elapsed().as_millis() / FRAME_MS as u128) as usize;
        st.spectrum.get(idx).copied().unwrap_or(SILENT)
    }
}

#[cfg(test)]
#[path = "../tests/unit/audio.rs"]
mod tests;
