
use super::*;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn be256(value: u64) -> [u8; 256] {
    let mut out = [0u8; 256];
    out[248..].copy_from_slice(&value.to_be_bytes());
    out
}

#[test]
fn sha1_matches_fips_180_vectors() {
    assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(
        hex(&sha1(b"abc")),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    // 56 bytes: the padding spills into a second 64-byte block.
    assert_eq!(
        hex(&sha1(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}

#[test]
fn limbs_round_trip() {
    let mut bytes = [0u8; 256];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = (i * 7 + 3) as u8;
    }
    assert_eq!(limbs_to_bytes(&bytes_to_limbs(&bytes)), bytes);
}

#[test]
fn modpow_small_numbers() {
    assert_eq!(
        rsa2048_modpow(&be256(2), &be256(10), &be256(1000)),
        be256(24)
    );
}
