//! M4A/AAC/ALAC decoding to interleaved PCM, safe to run on a blocking thread.

use std::io::Cursor;
use symphonia::core::{
    audio::{SampleBuffer, SignalSpec},
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use crate::error::{CalyxError, CalyxResult};
use crate::spectrum::{compute_spectrum_frames, SpectrumFrame};

#[derive(Debug)]
pub struct DecodedTrack {
    pub channels: u16,
    pub sample_rate: u32,
    pub pcm: Vec<i16>,
    pub spectrum: Vec<SpectrumFrame>,
}

pub fn decode_track(bytes: Vec<u8>) -> CalyxResult<DecodedTrack> {
    let (channels, sample_rate, pcm) = decode_m4a_to_pcm(bytes)?;
    let spectrum = compute_spectrum_frames(&pcm, channels, sample_rate);
    Ok(DecodedTrack { channels, sample_rate, pcm, spectrum })
}

thread_local! {
    static SILENCE_PANICS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Installs (once, process-wide) a panic hook that stays quiet only on threads
/// currently decoding, so concurrent decodes can't clobber the global hook.
fn install_quiet_panic_hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !SILENCE_PANICS.with(|s| s.get()) { prev(info); }
        }));
    });
}

fn decode_m4a_to_pcm(bytes: Vec<u8>) -> CalyxResult<(u16, u32, Vec<i16>)> {
    if bytes.is_empty() {
        return Err(CalyxError::Audio("Empty audio buffer".to_string()));
    }

    let mss = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("m4a");

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| CalyxError::Audio(format!("MP4/M4A probe error: {e}")))?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| CalyxError::Audio("No supported audio track in M4A container".to_string()))?;

    let track_id = track.id;
    let reserve = track.codec_params.n_frames.unwrap_or(0) as usize * track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| CalyxError::Audio(format!("AAC/ALAC codec init error: {e}")))?;

    let mut pcm: Vec<i16> = Vec::with_capacity(reserve);
    let (mut channels, mut sample_rate) = (2u16, 44_100u32);
    let mut sample_buf: Option<SampleBuffer<i16>> = None;
    let mut buf_spec: Option<SignalSpec> = None;
    let mut consecutive_errs = 0usize;

    install_quiet_panic_hook();
    // RAII: restores the flag even if a panic escapes, since tokio reuses blocking threads.
    struct QuietGuard;
    impl Drop for QuietGuard {
        fn drop(&mut self) { SILENCE_PANICS.with(|s| s.set(false)); }
    }
    SILENCE_PANICS.with(|s| s.set(true));
    let quiet = QuietGuard;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => { consecutive_errs = 0; p }
            Err(SymphoniaError::IoError(_)) => break,
            Err(SymphoniaError::ResetRequired) => { decoder.reset(); continue; }
            Err(_) => { consecutive_errs += 1; if consecutive_errs > 50 { break; } decoder.reset(); continue; }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let cap = decoded.capacity() as u64;
                    let needs_new = match (&sample_buf, buf_spec) {
                        (Some(sb), Some(s)) => s != spec || (sb.capacity() as u64) < cap * spec.channels.count() as u64,
                        _ => true,
                    };
                    if needs_new { sample_buf = Some(SampleBuffer::<i16>::new(cap, spec)); buf_spec = Some(spec); }
                    if let Some(sb) = sample_buf.as_mut() {
                        sb.copy_interleaved_ref(decoded);
                        pcm.extend_from_slice(sb.samples());
                    }
                    Some(spec)
                }
                Err(_) => None,
            }
        }));

        match res {
            Ok(Some(spec)) => { channels = (spec.channels.count() as u16).max(1); sample_rate = spec.rate.max(8_000); }
            Ok(None) => {}
            Err(_) => decoder.reset(),
        }
    }

    drop(quiet);

    if pcm.is_empty() {
        return Err(CalyxError::Audio("Decoded 0 PCM samples from stream".to_string()));
    }

    Ok((channels, sample_rate, pcm))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_bytes_is_error() {
        assert!(decode_track(Vec::new()).is_err());
    }
    #[test]
    fn garbage_bytes_is_error() {
        assert!(decode_track(vec![0u8; 64]).is_err());
    }
}