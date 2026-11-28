
use super::*;

#[test]
fn override_wins_over_platform() {
    let p = locate(
        Some(PathBuf::from("/portable")),
        Some(PathBuf::from("/xdg")),
        "config",
    );
    assert_eq!(p, PathBuf::from("/portable").join("config"));
}

#[test]
fn platform_dir_gets_app_subfolder() {
    assert_eq!(
        locate(None, Some(PathBuf::from("/xdg")), "config"),
        PathBuf::from("/xdg").join("calyx")
    );
    assert_eq!(locate(None, None, "config"), PathBuf::from("."));
}

#[test]
fn legacy_file_is_copied_once_and_kept() {
    let root = std::env::temp_dir().join(format!("calyx_paths_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let legacy = root.join("calyx_auth.json");
    let dest = root.join("new").join("calyx_auth.json");
    std::fs::write(&legacy, "old").unwrap();

    assert!(adopt_legacy_file(&legacy, &dest));
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "old");
    assert!(legacy.exists());

    std::fs::write(&legacy, "newer").unwrap();
    assert!(!adopt_legacy_file(&legacy, &dest));
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "old");

    let _ = std::fs::remove_dir_all(&root);
}
