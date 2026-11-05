use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use tracing::info;
use crate::decode::{decode_track, DecodedTrack};
use crate::error::{CalyxError, CalyxResult};
use crate::models::PlaybackStatus;
use crate::spectrum::{SpectrumFrame, FRAME_MS, SPECTRUM_BANDS};

const STATE_STOPPED: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PAUSED: u8 = 2;

pub struct NativeAudioEngine {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Mutex<Option<Sink>>,
    volume: Mutex<f32>,
    duration_secs: AtomicU64,
    started_at: Mutex<Option<std::time::Instant>>,
    accumulated_ms: AtomicU64,
    state: AtomicU8,
    spectrum_frames: Mutex<Vec<SpectrumFrame>>,
}

impl NativeAudioEngine {
    pub fn new() -> CalyxResult<Self> {
        let (_stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => { info!("Audio output device unavailable ({e}), running in headless mode"); (None, None) }
        };
        Ok(Self {
            _stream,
            handle,
            sink: Mutex::new(None),
            volume: Mutex::new(0.85),
            duration_secs: AtomicU64::new(0),
            started_at: Mutex::new(None),
            accumulated_ms: AtomicU64::new(0),
            state: AtomicU8::new(STATE_STOPPED),
            spectrum_frames: Mutex::new(Vec::with_capacity(128)),
        })
    }

    pub fn play_encoded_bytes(&self, bytes: Vec<u8>) -> CalyxResult<()> {
        self.play_decoded(decode_track(bytes)?)
    }

    pub fn play_decoded(&self, track: DecodedTrack) -> CalyxResult<()> {
        let DecodedTrack { channels, sample_rate, pcm, spectrum } = track;
        let dur_s = pcm.len() as u64 / (channels as u64 * sample_rate as u64).max(1);
        self.duration_secs.store(dur_s, Ordering::Relaxed);
        *self.spectrum_frames.lock().unwrap() = spectrum;

        if let Some(handle) = &self.handle {
            let new_sink = Sink::try_new(handle)
                .map_err(|e| CalyxError::Audio(format!("Failed to create audio sink: {e}")))?;
            let vol = *self.volume.lock().unwrap();
            new_sink.set_volume(vol);
            new_sink.append(SamplesBuffer::new(channels, sample_rate, pcm));
            new_sink.play();

            let mut guard = self.sink.lock().unwrap();
            if let Some(old) = guard.take() { old.stop(); }
            *guard = Some(new_sink);
        }
        self.accumulated_ms.store(0, Ordering::Relaxed);
        *self.started_at.lock().unwrap() = Some(std::time::Instant::now());
        self.state.store(STATE_PLAYING, Ordering::Relaxed);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        let current = self.state.load(Ordering::Relaxed);
        let guard = self.sink.lock().unwrap();
        if current == STATE_PLAYING {
            if let Some(sink) = guard.as_ref() { sink.pause(); }
            if let Some(t) = self.started_at.lock().unwrap().take() {
                self.accumulated_ms.fetch_add(t.elapsed().as_millis() as u64, Ordering::Relaxed);
            }
            self.state.store(STATE_PAUSED, Ordering::Relaxed);
        } else if current == STATE_PAUSED {
            if let Some(sink) = guard.as_ref() { sink.play(); }
            *self.started_at.lock().unwrap() = Some(std::time::Instant::now());
            self.state.store(STATE_PLAYING, Ordering::Relaxed);
        }
    }

    pub fn stop(&self) {
        *self.started_at.lock().unwrap() = None;
        self.accumulated_ms.store(0, Ordering::Relaxed);
        if let Some(sink) = self.sink.lock().unwrap().take() { sink.stop(); }
        self.state.store(STATE_STOPPED, Ordering::Relaxed);
    }

    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = clamped;
        if let Some(sink) = self.sink.lock().unwrap().as_ref() { sink.set_volume(clamped); }
    }

pub fn progress(&self) -> (Duration, Duration) {
    let dur = Duration::from_secs(self.duration_secs.load(Ordering::Relaxed));
    let live_ms = if self.state.load(Ordering::Relaxed) == STATE_PLAYING {
        self.started_at.lock().unwrap().map(|t| t.elapsed().as_millis() as u64).unwrap_or(0)
    } else { 0 };
    let elapsed = Duration::from_millis(self.accumulated_ms.load(Ordering::Relaxed) + live_ms);
    (elapsed, dur)
}

pub fn is_finished(&self) -> bool {
    if self.state.load(Ordering::Relaxed) != STATE_PLAYING {
        return false;
    }
    let guard = self.sink.lock().unwrap();
    let finished = guard.as_ref().map(|s| s.empty()).unwrap_or(false);
    if finished {
        *self.started_at.lock().unwrap() = None;
        self.state.store(STATE_STOPPED, Ordering::Relaxed);
    }
    finished
}

pub fn status(&self) -> PlaybackStatus {
    match self.state.load(Ordering::Relaxed) {
        STATE_PLAYING => PlaybackStatus::Playing,
        STATE_PAUSED => PlaybackStatus::Paused,
        _ => PlaybackStatus::Stopped,
    }
}

pub fn current_spectrum(&self) -> SpectrumFrame {
    const SILENT: SpectrumFrame = [0u8; SPECTRUM_BANDS];
    if self.state.load(Ordering::Relaxed) == STATE_STOPPED {
        return SILENT;
    }
    let idx = (self.progress().0.as_millis() / FRAME_MS as u128) as usize;
    let guard = self.spectrum_frames.lock().unwrap();
    guard.get(idx).copied().unwrap_or(SILENT)
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_audio_buffer_returns_error() {
        let engine = NativeAudioEngine {
            _stream: None,
            handle: None,
            sink: Mutex::new(None),
            volume: Mutex::new(0.85),
            duration_secs: AtomicU64::new(0),
            started_at: Mutex::new(None),
            accumulated_ms: AtomicU64::new(0),
            state: AtomicU8::new(STATE_STOPPED),
            spectrum_frames: Mutex::new(Vec::with_capacity(1024)),
        };
        let res = engine.play_encoded_bytes(Vec::new());
        assert!(res.is_err());
    }
}
