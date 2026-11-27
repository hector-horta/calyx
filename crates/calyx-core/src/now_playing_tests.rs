use super::*;
use serde_json::to_value;

fn sample() -> NowPlaying {
    NowPlaying {
        schema: SCHEMA,
        player: "calyx",
        state: PlayState::Playing,
        title: "Digital Love".into(),
        artist: "Daft Punk".into(),
        album: "Discovery".into(),
        artwork_url: None,
        duration_ms: 301_000,
        position_ms: 12_000,
        updated_at_ms: 1_700_000_000_000,
    }
}

fn temp_file(name: &str) -> PathBuf {
    // Its own folder: the test removes the parent dir.
    std::env::temp_dir()
        .join(format!("calyx_np_{}_{}", name, std::process::id()))
        .join("now_playing.json")
}

#[test]
fn the_json_has_the_documented_fields() {
    let value = to_value(sample()).unwrap();
    assert_eq!(value["schema"], 1);
    assert_eq!(value["player"], "calyx");
    assert_eq!(value["state"], "playing");
    assert_eq!(value["title"], "Digital Love");
    assert_eq!(value["duration_ms"], 301_000);
    assert_eq!(value["position_ms"], 12_000);
    assert!(value["artwork_url"].is_null());
}

#[test]
fn paused_is_lowercase() {
    let mut np = sample();
    np.state = PlayState::Paused;
    assert_eq!(to_value(np).unwrap()["state"], "paused");
}

#[test]
fn write_creates_the_dir_and_leaves_no_temp_file() {
    let path = temp_file("write");
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
    sample().write_to(&path).unwrap();
    let back: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(back["artist"], "Daft Punk");
    assert!(!path.with_extension("json.tmp").exists());
    clear(&path);
    assert!(!path.exists());
    clear(&path); // a missing file is fine
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn now_ms_is_after_2020() {
    assert!(now_ms() > 1_577_836_800_000);
}
