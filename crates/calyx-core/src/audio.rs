use std::io::Cursor;
use std::sync::atomic::{AtomicU8, Ordering};
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
    _handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    state: AtomicU8,
}

impl NativeAudioEngine {
    pub fn new() -> CalyxResult<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                let sink = Sink::try_new(&handle)
                    .map_err(|e| CalyxError::Audio(format!("Failed to create audio sink: {e}")))?;
                Ok(Self {
                    _stream: Some(stream),
                    _handle: Some(handle),
                    sink: Some(sink),
                    state: AtomicU8::new(STATE_STOPPED),
                })
            }
            Err(e) => {
                info!("Audio output device unavailable ({e}), running in headless mode");
                Ok(Self {
                    _stream: None,
                    _handle: None,
                    sink: None,
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

        if let Some(sink) = &self.sink {
            sink.stop();
            let source = SamplesBuffer::new(channels, sample_rate, pcm_samples);
            sink.append(source);
            sink.play();
        }
        self.state.store(STATE_PLAYING, Ordering::Relaxed);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        let current = self.state.load(Ordering::Relaxed);
        if current == STATE_PLAYING {
            if let Some(sink) = &self.sink {
                sink.pause();
            }
            self.state.store(STATE_PAUSED, Ordering::Relaxed);
        } else if current == STATE_PAUSED {
            if let Some(sink) = &self.sink {
                sink.play();
            }
            self.state.store(STATE_PLAYING, Ordering::Relaxed);
        }
    }

    pub fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.state.store(STATE_STOPPED, Ordering::Relaxed);
    }

    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(clamped);
        }
    }

    pub fn status(&self) -> PlaybackStatus {
        match self.state.load(Ordering::Relaxed) {
            STATE_PLAYING => PlaybackStatus::Playing,
            STATE_PAUSED => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
    }
}