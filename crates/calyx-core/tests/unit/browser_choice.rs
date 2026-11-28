
use super::*;

#[test]
fn unknown_names_find_nothing() {
    assert_eq!(named("netscape"), None);
    assert!(known_names().contains(&"safari"));
    assert!(known_names().contains(&"firefox"));
}

#[test]
fn the_app_bundle_is_found_above_the_executable() {
    let exe = Path::new("/Applications/Safari.app/Contents/MacOS/Safari");
    assert_eq!(app_bundle(exe), Some(Path::new("/Applications/Safari.app")));
    assert_eq!(app_bundle(Path::new("/usr/bin/epiphany")), None);
}
