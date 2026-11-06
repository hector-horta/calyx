//! Per-platform locations for calyx files, so the session and the history no
//! longer depend on the directory calyx is launched from.
//!
//! `CALYX_HOME` overrides every location (portable installs, tests).

use std::path::{Path, PathBuf};

const APP: &str = "calyx";

/// `override_root/sub` if set, else `platform/calyx`, else the current dir.
fn locate(override_root: Option<PathBuf>, platform: Option<PathBuf>, sub: &str) -> PathBuf {
    match (override_root, platform) {
        (Some(root), _) => root.join(sub),
        (None, Some(base)) => base.join(APP),
        (None, None) => PathBuf::from("."),
    }
}

fn home_override() -> Option<PathBuf> {
    std::env::var_os("CALYX_HOME").filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// Session and credentials: `%APPDATA%\calyx`, `~/Library/Application Support/calyx`, `~/.config/calyx`.
pub fn config_dir() -> PathBuf {
    locate(home_override(), dirs::config_dir(), "config")
}

/// Catalog database and the login browser profile.
pub fn data_dir() -> PathBuf {
    locate(home_override(), dirs::data_local_dir(), "data")
}

/// Decrypted audio cache. Kept at `~/.cache/calyx/audio` so existing caches survive.
pub fn audio_cache_dir() -> PathBuf {
    match home_override() {
        Some(root) => root.join("cache").join("audio"),
        None => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".cache").join(APP).join("audio"),
    }
}

pub fn auth_file() -> PathBuf {
    config_dir().join("calyx_auth.json")
}

pub fn catalog_db() -> PathBuf {
    data_dir().join("calyx_cache.db")
}

pub fn browser_profile_dir() -> PathBuf {
    data_dir().join("browser_profile")
}

/// Copies a file left by older versions (in the current directory) to its
/// new home, once. Never deletes or overwrites anything.
pub fn adopt_legacy_file(legacy: &Path, dest: &Path) -> bool {
    if dest.exists() || !legacy.is_file() {
        return false;
    }
    if let Some(parent) = dest.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    std::fs::copy(legacy, dest).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins_over_platform() {
        let p = locate(Some(PathBuf::from("/portable")), Some(PathBuf::from("/xdg")), "config");
        assert_eq!(p, PathBuf::from("/portable").join("config"));
    }

    #[test]
    fn platform_dir_gets_app_subfolder() {
        assert_eq!(locate(None, Some(PathBuf::from("/xdg")), "config"), PathBuf::from("/xdg").join("calyx"));
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
}