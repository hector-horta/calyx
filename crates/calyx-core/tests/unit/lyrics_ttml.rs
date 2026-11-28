
use super::*;

#[test]
fn test_parse_ttml_time() {
    assert_eq!(parse_ttml_time("1.319"), Some(1319));
    assert_eq!(parse_ttml_time("1:02.635"), Some(62635));
    assert_eq!(parse_ttml_time("01:02.635"), Some(62635));
    assert_eq!(parse_ttml_time("1:02:03.500"), Some(3723500));
}

#[test]
fn test_parse_ttml_japanese_sample() {
    let sample = r#"<tt xml:lang="ja"><body><div>
            <p begin="1.319" end="2.555">吠えるわ</p>
            <p begin="1:02.635" end="1:10.372">心静かに騒ぐ &amp; <span>抱きしめたい</span></p>
        </div></body></tt>"#;
    let lines = parse_ttml(sample);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].time_ms, 1319);
    assert_eq!(lines[0].text, "吠えるわ");
    assert_eq!(lines[1].time_ms, 62635);
    assert_eq!(lines[1].text, "心静かに騒ぐ & 抱きしめたい");
}
