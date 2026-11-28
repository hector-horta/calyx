
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
    let count =
        decrypt_fmp4_segment_in_place(&mut fake_segment, &decryptor).expect("decrypt segment");
    assert_eq!(count, 1);
    assert_eq!(&fake_segment[12..16], b"mp4a");
    assert_ne!(&fake_segment[24..40], &[0xAAu8; 16]);
}
