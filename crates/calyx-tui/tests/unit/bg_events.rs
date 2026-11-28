use super::*;

fn audio(source: AudioSource, note: Option<&str>) -> ResolvedAudio {
    ResolvedAudio {
        bytes: Vec::new(),
        source,
        note: note.map(str::to_string),
    }
}

#[test]
fn hint_reflects_where_audio_came_from() {
    use calyx_core::Language::{En, Es};
    assert_eq!(
        audio_hint(&audio(AudioSource::Cache, None), Es),
        "[AAC 256k | Caché]"
    );
    assert_eq!(
        audio_hint(&audio(AudioSource::Cache, None), En),
        "[AAC 256k | Cached]"
    );
    assert_eq!(
        audio_hint(&audio(AudioSource::FullStream, None), Es),
        "[AAC 256k]"
    );
    assert_eq!(
        audio_hint(&audio(AudioSource::Preview, Some("HTTP 403")), Es),
        "[Preview 30s | HTTP 403]"
    );
    assert!(audio_hint(&audio(AudioSource::Preview, None), En).contains("log in"));
}
