use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use crate::drm::CbcsDecryptor;
use crate::error::{CalyxError, CalyxResult};

type Aes128Ctr = ctr::Ctr128BE<Aes128>;

pub const DEFAULT_PREFETCH_KEY: [u8; 16] = [
    0x32, 0xb8, 0xad, 0xe1, 0x76, 0x9e, 0x26, 0xb1, 0xff, 0xb8, 0x98, 0x63, 0x52, 0x79, 0x3f, 0xc6,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4BoxHeader {
    pub box_type: [u8; 4],
    pub offset: usize,
    pub payload_offset: usize,
    pub size: usize,
}

pub fn scan_mp4_boxes(data: &[u8]) -> Vec<Mp4BoxHeader> {
    let mut boxes = Vec::new();
    let mut cursor = 0usize;

    while cursor + 8 <= data.len() {
        let raw_size = u32::from_be_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        let box_type = [
            data[cursor + 4],
            data[cursor + 5],
            data[cursor + 6],
            data[cursor + 7],
        ];

        let box_size = if raw_size == 0 {
            data.len() - cursor
        } else if raw_size < 8 || cursor + raw_size > data.len() {
            break;
        } else {
            raw_size
        };

        boxes.push(Mp4BoxHeader {
            box_type,
            offset: cursor,
            payload_offset: cursor + 8,
            size: box_size,
        });

        cursor += box_size;
    }
    boxes
}

pub fn decrypt_apple_fmp4_stream(
    stream: &mut [u8],
    track_key: &[u8; 16],
    use_cenc: bool,
) -> CalyxResult<usize> {
    let is_cenc = use_cenc || stream.windows(4).any(|w| w == b"senc" || w == b"cenc");
    if !is_cenc {
        let cbcs = CbcsDecryptor::new(*track_key, [0u8; 16]);
        return decrypt_fmp4_segment_in_place(stream, &cbcs);
    }
    let boxes = scan_mp4_boxes(stream);
    let mut decrypted_samples = 0usize;
    let mut idx = 0usize;
    while idx < boxes.len() {
        if &boxes[idx].box_type == b"moof" && idx + 1 < boxes.len() && &boxes[idx + 1].box_type == b"mdat" {
            let moof_slice = &stream[boxes[idx].offset..boxes[idx].offset + boxes[idx].size];
            let (_desc_index, sizes, ivs, all_subs) = parse_moof_cenc_meta(moof_slice);
            let key = track_key;
            let mdat_start = boxes[idx + 1].payload_offset;
            let mdat_end = boxes[idx + 1].offset + boxes[idx + 1].size;
            if mdat_start < mdat_end && mdat_end <= stream.len() {
                let mdat = &mut stream[mdat_start..mdat_end];
                let mut pos = 0usize;
                for (s_idx, &sz) in sizes.iter().enumerate() {
                    if sz == 0 || pos + sz > mdat.len() { break; }
                    let sample_start = pos;
                    let iv = ivs.get(s_idx).copied().unwrap_or([0u8; 16]);
                    let mut cipher = Aes128Ctr::new(key.into(), (&iv).into());
                    if let Some(subs) = all_subs.get(s_idx).filter(|s| !s.is_empty()) {
                        let mut sub_pos = sample_start;
                        for &(clear, prot) in subs {
                            sub_pos += clear as usize;
                            let p = prot as usize;
                            if sub_pos + p <= mdat.len() { cipher.apply_keystream(&mut mdat[sub_pos..sub_pos + p]); sub_pos += p; }
                        }
                    } else {
                        cipher.apply_keystream(&mut mdat[sample_start..sample_start + sz]);
                    }
                    pos = sample_start + sz;
                    decrypted_samples += 1;
                }
            }
            idx += 2;
        } else {
            idx += 1;
        }
    }
    strip_enca_protection_atoms(stream);
    Ok(decrypted_samples)
}

fn parse_moof_cenc_meta(moof: &[u8]) -> (u32, Vec<usize>, Vec<[u8; 16]>, Vec<Vec<(u16, u32)>>) {
    let mut desc_index = 2u32;
    let mut default_size = 0usize;
    let mut sizes = Vec::new();
    let mut raw_senc: Option<&[u8]> = None;

    for b in scan_mp4_boxes(moof.get(8..).unwrap_or(&[])) {
        if &b.box_type == b"traf" {
            let traf = &moof[8 + b.offset..8 + b.offset + b.size];
            for tb in scan_mp4_boxes(traf.get(8..).unwrap_or(&[])) {
                let payload = &traf[8 + tb.payload_offset..8 + tb.offset + tb.size];
                if &tb.box_type == b"tfhd" && payload.len() >= 8 {
                    let flags = u32::from_be_bytes([0, payload[1], payload[2], payload[3]]);
                    let mut off = 8usize;
                    if flags & 0x01 != 0 { off += 8; }
                    if flags & 0x02 != 0 && off + 4 <= payload.len() {
                        desc_index = u32::from_be_bytes(payload[off..off + 4].try_into().unwrap_or([0, 0, 0, 2]));
                        off += 4;
                    }
                    if flags & 0x08 != 0 { off += 4; }
                    if flags & 0x10 != 0 && off + 4 <= payload.len() {
                        default_size = u32::from_be_bytes(payload[off..off + 4].try_into().unwrap_or([0; 4])) as usize;
                    }
                } else if &tb.box_type == b"trun" && payload.len() >= 8 {
                    let flags = u32::from_be_bytes([0, payload[1], payload[2], payload[3]]);
                    let count = u32::from_be_bytes(payload[4..8].try_into().unwrap_or([0; 4])) as usize;
                    let mut off = 8usize;
                    if flags & 0x01 != 0 { off += 4; }
                    if flags & 0x04 != 0 { off += 4; }
                    for _ in 0..count.min(4096) {
                        if flags & 0x100 != 0 { off += 4; }
                        let sz = if flags & 0x200 != 0 && off + 4 <= payload.len() {
                            let v = u32::from_be_bytes(payload[off..off + 4].try_into().unwrap_or([0; 4])) as usize;
                            off += 4;
                            v
                        } else {
                            default_size
                        };
                        if flags & 0x400 != 0 { off += 4; }
                        if flags & 0x800 != 0 { off += 4; }
                        sizes.push(sz);
                    }
                } else if &tb.box_type == b"senc" {
                    raw_senc = Some(payload);
                }
            }
        }
    }
    let mut ivs = Vec::with_capacity(sizes.len());
    let mut all_subs = Vec::with_capacity(sizes.len());
    if let Some(senc) = raw_senc {
        if senc.len() >= 8 {
            let flags = u32::from_be_bytes([0, senc[1], senc[2], senc[3]]);
            let has_sub = (flags & 0x02) != 0;
            let body = &senc[8..];
            let mut off = 0usize;
            for _ in 0..sizes.len() {
                if off + 8 > body.len() { break; }
                let mut iv = [0u8; 16];
                iv[..8].copy_from_slice(&body[off..off + 8]);
                ivs.push(iv);
                off += 8;
                let mut subs = Vec::new();
                if has_sub && off + 2 <= body.len() {
                    let sub_cnt = u16::from_be_bytes([body[off], body[off + 1]]) as usize;
                    off += 2;
                    for _ in 0..sub_cnt {
                        if off + 6 > body.len() { break; }
                        subs.push((u16::from_be_bytes([body[off], body[off + 1]]), u32::from_be_bytes([body[off + 2], body[off + 3], body[off + 4], body[off + 5]])));
                        off += 6;
                    }
                }
                all_subs.push(subs);
            }
        }
    }
    (desc_index, sizes, ivs, all_subs)
}

pub fn decrypt_fmp4_segment_in_place(
    segment: &mut [u8],
    decryptor: &CbcsDecryptor,
) -> CalyxResult<usize> {
    if segment.len() < 16 {
        return Err(CalyxError::Drm("fMP4 segment too small".to_string()));
    }

    strip_enca_protection_atoms(segment);
    let boxes = scan_mp4_boxes(segment);
    let mut decrypted_mdat_count = 0usize;

    for box_header in &boxes {
        if &box_header.box_type == b"mdat" {
            let end = box_header.offset + box_header.size;
            if box_header.payload_offset < end && end <= segment.len() {
                decryptor.decrypt_sample_in_place(
                    &mut segment[box_header.payload_offset..end],
                    1,
                    0,
                )?;
                decrypted_mdat_count += 1;
            }
        }
    }

    Ok(decrypted_mdat_count)
}

pub fn strip_enca_protection_atoms(buffer: &mut [u8]) {
    let limit = buffer.windows(4).position(|w| w == b"mdat").unwrap_or(buffer.len());
    if limit < 8 {
        return;
    }
    for i in 0..limit.saturating_sub(4) {
        if &buffer[i..i + 4] == b"enca" {
            buffer[i..i + 4].copy_from_slice(b"mp4a");
        } else if &buffer[i..i + 4] == b"sinf" || &buffer[i..i + 4] == b"senc" {
            buffer[i..i + 4].copy_from_slice(b"free");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmp4_mdat_decryption_and_enca_strip() {
        let mut fake_segment = Vec::new();
        fake_segment.extend_from_slice(&16u32.to_be_bytes());
        fake_segment.extend_from_slice(b"moof");
        fake_segment.extend_from_slice(b"testenca");

        fake_segment.extend_from_slice(&24u32.to_be_bytes());
        fake_segment.extend_from_slice(b"mdat");
        fake_segment.extend_from_slice(&[0xAAu8; 16]);

        let decryptor = CbcsDecryptor::new([0x11; 16], [0x22; 16]);
        let count = decrypt_fmp4_segment_in_place(&mut fake_segment, &decryptor)
            .expect("decrypt segment");
        assert_eq!(count, 1);
        assert_eq!(&fake_segment[12..16], b"mp4a");
        assert_ne!(&fake_segment[24..40], &[0xAAu8; 16]);
    }
}
