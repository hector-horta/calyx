use super::*;

/// A fresh folder of its own: the test removes it at the end.
fn temp_log(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("calyx_log_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("calyx.log")
}

#[test]
fn a_small_log_stays_in_place() {
    let log = temp_log("small");
    std::fs::write(&log, "12345").unwrap();
    rotate(&log, 10);
    assert!(log.exists());
    assert!(!log.with_extension("log.1").exists());
    let _ = std::fs::remove_dir_all(log.parent().unwrap());
}

#[test]
fn a_big_log_replaces_the_previous_one() {
    let log = temp_log("big");
    let old = log.with_extension("log.1");
    std::fs::write(&old, "older").unwrap();
    std::fs::write(&log, "0123456789ABC").unwrap();
    rotate(&log, 10);
    assert!(!log.exists());
    assert_eq!(std::fs::read_to_string(&old).unwrap(), "0123456789ABC");
    let _ = std::fs::remove_dir_all(log.parent().unwrap());
}

#[test]
fn a_missing_log_is_fine() {
    let log = temp_log("missing");
    rotate(&log, 10);
    assert!(!log.exists());
    let _ = std::fs::remove_dir_all(log.parent().unwrap());
}
