use super::*;
#[test]
fn empty_bytes_is_error() {
    assert!(decode_track(Vec::new()).is_err());
}
#[test]
fn garbage_bytes_is_error() {
    assert!(decode_track(vec![0u8; 64]).is_err());
}
