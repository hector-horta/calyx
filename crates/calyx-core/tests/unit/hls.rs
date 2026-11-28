use super::*;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

#[test]
fn test_parse_apple_hls_m3u8() {
    let mut raw_pssh = vec![0u8; 32];
    raw_pssh[4..8].copy_from_slice(b"pssh");
    let b64_pssh = BASE64.encode(&raw_pssh);

    let m3u8 = format!(
        "#EXTM3U\n\
             #EXT-X-VERSION:7\n\
             #EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"data:text/plain;base64,{}\",KEYFORMAT=\"urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed\"\n\
             #EXT-X-MAP:URI=\"init_0.mp4\"\n\
             #EXTINF:4.000,\n\
             seg_0001.m4s\n\
             #EXTINF:4.000,\n\
             seg_0002.m4s\n",
        b64_pssh
    );

    let plan = HlsStreamPlan::parse_m3u8(
        &m3u8,
        "https://aod-ssl.itunes.apple.com/streams/master.m3u8",
    )
    .expect("valid m3u8");
    assert!(plan.pssh.is_some());
    assert_eq!(
        plan.init_segment_url.as_deref(),
        Some("https://aod-ssl.itunes.apple.com/streams/init_0.mp4")
    );
    assert_eq!(plan.media_segment_urls.len(), 2);

    let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=256000,CODECS=\"mp4a.40.2\"\naudio_256/prog_index.m3u8\n";
    assert_eq!(
        HlsStreamPlan::extract_variant_url(
            master,
            "https://aod-ssl.itunes.apple.com/streams/master.m3u8"
        )
        .as_deref(),
        Some("https://aod-ssl.itunes.apple.com/streams/audio_256/prog_index.m3u8")
    );
}
