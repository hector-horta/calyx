use super::*;

#[test]
fn test_parse_binary_cookies_synthetic() {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"cook");
    buf.extend_from_slice(&1u32.to_be_bytes()); // 1 page
    let mut page = vec![0x00, 0x00, 0x01, 0x00];
    page.extend_from_slice(&1u32.to_le_bytes()); // 1 cookie
    page.extend_from_slice(&12u32.to_le_bytes()); // offset = 12

    let mut cookie = vec![0u8; 32];
    let mut data = Vec::new();
    let u_off = 32 + data.len();
    data.extend_from_slice(b".music.apple.com\0");
    let n_off = 32 + data.len();
    data.extend_from_slice(b"media-user-token\0");
    let v_off = 32 + data.len();
    data.extend_from_slice(b"safari_test_123\0");
    let total_size = 32 + data.len();

    cookie[0..4].copy_from_slice(&(total_size as u32).to_le_bytes());
    cookie[16..20].copy_from_slice(&(u_off as u32).to_le_bytes());
    cookie[20..24].copy_from_slice(&(n_off as u32).to_le_bytes());
    cookie[28..32].copy_from_slice(&(v_off as u32).to_le_bytes());
    cookie.extend_from_slice(&data);

    page.extend_from_slice(&cookie);
    buf.extend_from_slice(&(page.len() as u32).to_be_bytes());
    buf.extend_from_slice(&page);

    assert_eq!(
        parse_binary_cookies(&buf),
        Some("safari_test_123".to_string())
    );
}
