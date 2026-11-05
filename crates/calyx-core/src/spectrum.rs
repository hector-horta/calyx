//! Precomputed pseudo-spectrum frames used by the TUI visualizer.
//!
//! Levels are in dB relative to each band's own 95th percentile across the
//! track, so loud masters and quiet recordings both use the full bar height.

pub const SPECTRUM_BANDS: usize = 12;
pub const FRAME_MS: u64 = 50;
/// Maximum bar level (3 rows x 8 eighth-blocks).
pub const MAX_LEVEL: u8 = 24;
/// Dynamic range shown on screen, in dB below the band reference.
const RANGE_DB: f32 = 30.0;
/// Max level drop per frame (peak fall-off); rises are immediate.
const FALL_PER_FRAME: u8 = 3;

pub type SpectrumFrame = [u8; SPECTRUM_BANDS];

fn raw_energies(pcm: &[i16], channels: u16, sample_rate: u32) -> Vec<[u32; SPECTRUM_BANDS]> {
    let ch = channels.max(1) as usize;
    let frames_per_win = (sample_rate as usize / 20).max(32);
    let win = frames_per_win * ch;
    let mut out = Vec::with_capacity(pcm.len().div_ceil(win));
    let mut mono: Vec<i32> = Vec::with_capacity(frames_per_win);

    for chunk in pcm.chunks(win) {
        mono.clear();
        for frame in chunk.chunks(ch) {
            if frame.is_empty() { continue; }
            mono.push(frame.iter().map(|&s| s as i32).sum::<i32>() / frame.len() as i32);
        }

        let mut row = [0u32; SPECTRUM_BANDS];
        let sub_len = (mono.len() / SPECTRUM_BANDS).max(1);

        for (b, sub) in mono.chunks(sub_len).take(SPECTRUM_BANDS).enumerate() {
            if sub.is_empty() { row[b] = 0; continue; }

            let b32 = (b * 32 / SPECTRUM_BANDS) as u64;
            let step = (32 - b32 as usize).clamp(1, 16);
            let mut sum = 0u64;
            let mut diff = 0u64;
            let mut last = sub[0];

            for &s in sub.iter().step_by(step) {
                sum += s.unsigned_abs() as u64;
                diff += (s - last).unsigned_abs() as u64;
                last = s;
            }

            let n = (sub.len() / step).max(1) as u64;
            row[b] = (((sum / n) * (32 - b32) + (diff / n) * (b32 + 6)) / 24) as u32;
        }

        out.push(row);
    }

    out
}

fn band_references(raw: &[[u32; SPECTRUM_BANDS]]) -> [f32; SPECTRUM_BANDS] {
    let mut refs = [1.0f32; SPECTRUM_BANDS];
    let mut v: Vec<u32> = Vec::with_capacity(raw.len());
    for (b, slot) in refs.iter_mut().enumerate() {
        v.clear();
        v.extend(raw.iter().map(|r| r[b]));
        if v.is_empty() { continue; }

        let k = (v.len() * 95 / 100).min(v.len() - 1);
        let (_, p, _) = v.select_nth_unstable(k);
        *slot = (*p as f32).max(1.0);
    }
    refs
}

fn level_db(energy: u32, reference: f32) -> u8 {
    if energy == 0 { return 1; }

    let db = 20.0 * (energy as f32 / reference).log10();
    let t = ((db + RANGE_DB) / RANGE_DB).clamp(0.0, 1.0);
    (1.0 + t * (MAX_LEVEL as f32 - 1.0)).round() as u8
}

pub fn compute_spectrum_frames(pcm: &[i16], channels: u16, sample_rate: u32) -> Vec<SpectrumFrame> {
    let raw = raw_energies(pcm, channels, sample_rate);
    if raw.is_empty() { return Vec::new(); }

    let refs = band_references(&raw);

    let mut prev: SpectrumFrame = [0u8; SPECTRUM_BANDS];
    raw.iter().map(|r| {
        let mut row: SpectrumFrame = [0u8; SPECTRUM_BANDS];
        for b in 0..SPECTRUM_BANDS {
            row[b] = level_db(r[b], refs[b]).max(prev[b].saturating_sub(FALL_PER_FRAME));
        }
        prev = row;
        row
    }).collect()
}
#[cfg(test)]
#[path = "spectrum_tests.rs"]
mod tests;
