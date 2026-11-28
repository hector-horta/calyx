
use super::*;
#[test]
fn test_parse_lrc() {
    let out = parse_lrc("[00:12.50] Primera línea\n[01:02.00] Segunda línea");
    assert_eq!(out.len(), 3);
    assert_eq!(out[0].text, "•••");
    assert_eq!(out[1].time_ms, 12_500);
    let (s, a, u) = out[1].karaoke_parts(13_500, 16_500, 40);
    assert!(!s.is_empty() || !a.is_empty() || !u.is_empty());
}
