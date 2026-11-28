use super::*;

#[test]
fn ws_url_must_be_loopback_with_port() {
    assert_eq!(
        split_ws_url("ws://127.0.0.1:45123/devtools/page/AB"),
        Some((45123, "/devtools/page/AB"))
    );
    assert_eq!(
        split_ws_url("ws://evil.example:9222/devtools/page/AB"),
        None
    );
    assert_eq!(split_ws_url("ws://127.0.0.1:notaport/x"), None);
}

#[test]
fn free_port_is_nonzero() {
    assert!(free_local_port().is_some_and(|p| p != 0));
}

#[test]
fn devtools_port_is_read_from_profile() {
    let dir = std::env::temp_dir().join(format!("calyx_cdp_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    assert_eq!(profile_devtools_port(&dir), None);
    std::fs::write(
        dir.join("DevToolsActivePort"),
        "51234\n/devtools/browser/x\n",
    )
    .unwrap();
    assert_eq!(profile_devtools_port(&dir), Some(51234));
    let _ = std::fs::remove_dir_all(&dir);
}
