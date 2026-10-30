use std::io::Cursor;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use tracing::info;
use crate::error::{CalyxError, CalyxResult};
use crate::models::PlaybackStatus;

const STATE_STOPPED: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PAUSED: u8 = 2;

pub struct NativeAudioEngine {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Mutex<Option<Sink>>,
    volume: Mutex<f32>,
    duration_secs: AtomicU64,
    state: AtomicU8,
}

impl NativeAudioEngine {
    pub fn new() -> CalyxResult<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                Ok(Self {
                    _stream: Some(stream),
                    handle: Some(handle),
                    sink: Mutex::new(None),
                    volume: Mutex::new(0.85),
                    duration_secs: AtomicU64::new(0),
                    state: AtomicU8::new(STATE_STOPPED),
                })
            }
            Err(e) => {
                info!("Audio output device unavailable ({e}), running in headless mode");
                Ok(Self {
                    _stream: None,
                    handle: None,
                    sink: Mutex::new(None),
                    volume: Mutex::new(0.85),
                    duration_secs: AtomicU64::new(0),
                    state: AtomicU8::new(STATE_STOPPED),
                })
            }
        }
    }

    pub fn play_encoded_bytes(&self, bytes: Vec<u8>) -> CalyxResult<()> {
        if bytes.is_empty() {
            return Err(CalyxError::Audio("Empty audio buffer".to_string()));
        }

        let cursor = Cursor::new(bytes);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("m4a");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| CalyxError::Audio(format!("MP4/M4A probe error: {e}")))?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| CalyxError::Audio("No supported audio track in M4A container".to_string()))?;

        let track_id = track.id;
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| CalyxError::Audio(format!("AAC/ALAC codec init error: {e}")))?;

        let mut pcm_samples: Vec<i16> = Vec::new();
        let mut channels: u16 = 2;
        let mut sample_rate: u32 = 44_100;

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(SymphoniaError::IoError(_)) => break,
                Err(SymphoniaError::ResetRequired) => {
                    decoder.reset();
                    continue;
                }
                Err(_) => break,
            };

            if packet.track_id() != track_id {
                continue;
            }

            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let decoded_samples = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match decoder.decode(&packet) {
                    Ok(decoded) => {
                        let spec = *decoded.spec();
                        let mut sample_buf = SampleBuffer::<i16>::new(decoded.capacity() as u64, spec);
                        sample_buf.copy_interleaved_ref(decoded);
                        Some((spec.channels.count() as u16, spec.rate, sample_buf.samples().to_vec()))
                    }
                    Err(_) => None,
                }
            }));
            std::panic::set_hook(prev_hook);

            match decoded_samples {
                Ok(Some((ch, rate, samples))) => {
                    channels = ch.max(1);
                    sample_rate = rate.max(8_000);
                    pcm_samples.extend_from_slice(&samples);
                }
                Ok(None) => {}
                Err(_) => {
                    decoder.reset();
                }
            }
        }

        if pcm_samples.is_empty() {
            return Err(CalyxError::Audio("Decoded 0 PCM samples from stream".to_string()));
        }

        let dur_s = pcm_samples.len() as u64 / (channels as u64 * sample_rate as u64).max(1);
        self.duration_secs.store(dur_s, Ordering::Relaxed);

        if let Some(handle) = &self.handle {
            let new_sink = Sink::try_new(handle)
                .map_err(|e| CalyxError::Audio(format!("Failed to create audio sink: {e}")))?;
            let vol = *self.volume.lock().unwrap();
            new_sink.set_volume(vol);
            let source = SamplesBuffer::new(channels, sample_rate, pcm_samples);
            new_sink.append(source);
            new_sink.play();

            let mut guard = self.sink.lock().unwrap();
            if let Some(old) = guard.take() {
                old.stop();
            }
            *guard = Some(new_sink);
        }

        self.state.store(STATE_PLAYING, Ordering::Relaxed);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        let current = self.state.load(Ordering::Relaxed);
        let guard = self.sink.lock().unwrap();
        if current == STATE_PLAYING {
            if let Some(sink) = guard.as_ref() { sink.pause(); }
            self.state.store(STATE_PAUSED, Ordering::Relaxed);
        } else if current == STATE_PAUSED {
            if let Some(sink) = guard.as_ref() { sink.play(); }
            self.state.store(STATE_PLAYING, Ordering::Relaxed);
        }
    }

    pub fn stop(&self) {
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
        let elapsed = self.sink.lock().unwrap()
            .as_ref()
            .map(|s| s.get_pos())
            .unwrap_or(Duration::ZERO);
        (elapsed, dur)
    }

    pub fn status(&self) -> PlaybackStatus {
        match self.state.load(Ordering::Relaxed) {
            STATE_PLAYING => PlaybackStatus::Playing,
            STATE_PAUSED => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
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
            state: AtomicU8::new(STATE_STOPPED),
        };
        let res = engine.play_encoded_bytes(Vec::new());
        assert!(res.is_err());
    }
}