//! Precomputed pseudo-spectrum frames used by the TUI visualizer.

pub const SPECTRUM_BANDS: usize = 12;
pub const FRAME_MS: u64 = 50;

pub type SpectrumFrame = [u8; SPECTRUM_BANDS];

pub fn compute_spectrum_frames(pcm: &[i16], channels: u16, sample_rate: u32) -> Vec<SpectrumFrame> {
    let ch = channels.max(1) as usize;
    let frames_per_win = (sample_rate as usize / 20).max(32);
    let win = frames_per_win * ch;
    let mut out = Vec::with_capacity(pcm.len() / win + 1);

    let mut prev: SpectrumFrame = [0u8; SPECTRUM_BANDS];
    let mut mono: Vec<i32> = Vec::with_capacity(frames_per_win);

    for chunk in pcm.chunks(win) {
        mono.clear();
        for frame in chunk.chunks(ch) {
            let sum = frame.iter().map(|&s| s as i32).sum::<i32>();
            mono.push(sum / frame.len() as i32);
        }

        let mut row: SpectrumFrame = [0u8; SPECTRUM_BANDS];
        let sub_len = (mono.len() / SPECTRUM_BANDS).max(1);

        for (b, sub) in mono.chunks(sub_len).take(SPECTRUM_BANDS).enumerate() {
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
            let energy = ((sum / n) * (32 - b32) + (diff / n) * (b32 + 6)) / 24;
            let scaled = (energy / 210).clamp(1, 24) as u16;
            row[b] = ((prev[b] as u16 * 2 + scaled * 3) / 5) as u8;
            prev[b] = row[b];
        }

        out.push(row);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_no_frames() {
        assert!(compute_spectrum_frames(&[], 2, 44_100).is_empty());
    }

    #[test]
    fn frame_count_matches_windows() {
        let pcm = vec![0i16; 44_100 * 2];
        assert_eq!(compute_spectrum_frames(&pcm, 2, 44_100).len(), 20);
    }

    #[test]
    fn identical_stereo_equals_mono() {
        let mono = (0..44_100)
            .map(|i| ((i as i32 * 37) % 2000 - 1000) as i16)
            .collect::<Vec<_>>();

        let stereo: Vec<i16> = mono.iter().flat_map(|&s| [s, s]).collect();
        assert_eq!(compute_spectrum_frames(&mono, 1, 44_100), compute_spectrum_frames(&stereo, 2, 44_100));
    }
}