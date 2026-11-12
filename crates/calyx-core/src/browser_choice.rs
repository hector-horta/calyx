//! The login browser picked on the command line (`calyx -u firefox`).
//! When set it wins over the automatic discovery, and the Firefox/WebKit
//! flows open the login page in it instead of the system default browser.

use crate::browser_discovery::BrowserType;
use crate::error::{CalyxError, CalyxResult};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PREFERRED: OnceLock<BrowserType> = OnceLock::new();

/// Names accepted by `-u`: (name, family, executables looked up in PATH,
/// install paths relative to Program Files / LocalAppData / Applications).
type Known = (
    &'static str,
    fn(PathBuf) -> BrowserType,
    &'static [&'static str],
    &'static [&'static str],
);

const KNOWN: &[Known] = &[
    (
        "chrome",
        BrowserType::Chromium,
        &["google-chrome", "google-chrome-stable", "chrome"],
        &[
            r"Google\Chrome\Application\chrome.exe",
            "Google Chrome.app/Contents/MacOS/Google Chrome",
        ],
    ),
    (
        "chromium",
        BrowserType::Chromium,
        &["chromium", "chromium-browser"],
        &["Chromium.app/Contents/MacOS/Chromium"],
    ),
    (
        "edge",
        BrowserType::Chromium,
        &["msedge", "microsoft-edge"],
        &[
            r"Microsoft\Edge\Application\msedge.exe",
            "Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ],
    ),
    (
        "brave",
        BrowserType::Chromium,
        &["brave-browser", "brave"],
        &[
            r"BraveSoftware\Brave-Browser\Application\brave.exe",
            "Brave Browser.app/Contents/MacOS/Brave Browser",
        ],
    ),
    (
        "arc",
        BrowserType::Chromium,
        &["arc"],
        &["Arc.app/Contents/MacOS/Arc"],
    ),
    (
        "firefox",
        BrowserType::Firefox,
        &["firefox"],
        &[
            r"Mozilla Firefox\firefox.exe",
            "Firefox.app/Contents/MacOS/firefox",
        ],
    ),
    (
        "librewolf",
        BrowserType::Firefox,
        &["librewolf"],
        &[
            r"LibreWolf\librewolf.exe",
            "LibreWolf.app/Contents/MacOS/librewolf",
        ],
    ),
    (
        "zen",
        BrowserType::Firefox,
        &["zen", "zen-browser"],
        &[r"Zen Browser\zen.exe", "Zen.app/Contents/MacOS/zen"],
    ),
    (
        "safari",
        BrowserType::WebKit,
        &[],
        &["Safari.app/Contents/MacOS/Safari"],
    ),
    (
        "orion",
        BrowserType::WebKit,
        &[],
        &["Orion.app/Contents/MacOS/Orion"],
    ),
    ("epiphany", BrowserType::WebKit, &["epiphany"], &[]),
];

/// Names `-u` accepts, for the error message.
pub fn known_names() -> Vec<&'static str> {
    KNOWN.iter().map(|k| k.0).collect()
}

/// Uses `browser` for every login from now on (set once, at startup).
pub fn prefer(browser: BrowserType) {
    let _ = PREFERRED.set(browser);
}

pub fn preferred() -> Option<&'static BrowserType> {
    PREFERRED.get()
}

/// Folders browsers are installed under on this machine.
fn install_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"]
        .iter()
        .filter_map(|v| std::env::var_os(v).map(PathBuf::from))
        .collect();
    roots.push(PathBuf::from("/Applications"));
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join("Applications"));
    }
    roots
}

/// Returns the `BrowserType` for a named browser, if found.
pub fn named(name: &str) -> Option<BrowserType> {
    let wanted = name.trim().to_lowercase();
    let (_, make, path_names, installs) = KNOWN.iter().find(|k| k.0 == wanted)?;

    for exe in path_names.iter() {
        if let Some(p) = crate::browser_discovery::find_in_path(exe) {
            return Some(make(p));
        }
    }

    for root in install_roots() {
        for rel in installs.iter() {
            let p = root.join(rel);
            if p.is_file() {
                return Some(make(p));
            }
        }
    }

    None
}

/// Opens the login page URL in the preferred browser or system default if none is set.
pub fn open_login_page(url: &str) -> CalyxResult<()> {
    let exe = match preferred() {
        Some(BrowserType::Firefox(p)) | Some(BrowserType::WebKit(p)) => p,
        _ => return crate::login_firefox::open_url_in_system_browser(url),
    };
    let mut cmd = match app_bundle(exe) {
        Some(app) if cfg!(target_os = "macos") => {
            let mut c = std::process::Command::new("open");
            c.arg("-a").arg(app).arg(url);
            c
        }
        _ => {
            let mut c = std::process::Command::new(exe);
            c.arg(url);
            c
        }
    };
    cmd.spawn()
        .map_err(|e| CalyxError::Auth(format!("No se pudo abrir {}: {e}", exe.display())))?;
    Ok(())
}

/// macOS app bundle holding `exe` (`/Applications/Safari.app`), if any.
fn app_bundle(exe: &Path) -> Option<&Path> {
    exe.ancestors()
        .find(|p| p.extension().is_some_and(|e| e == "app"))
}

#[cfg(test)]
mod tests {
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
}
