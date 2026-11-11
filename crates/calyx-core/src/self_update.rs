//! Installs a newer calyx release over the running binary, on every OS.
//!
//! The archive comes from the GitHub release (URL rebuilt from the parsed
//! version), is checked against the `.sha256` file published next to it, and
//! only its `calyx`/`calyx.exe` entry is extracted, as `calyx.new` beside the
//! running binary. Windows cannot overwrite or delete a running exe but can
//! rename it, so the swap is: `calyx.exe` → `calyx.old`, `calyx.new` →
//! `calyx.exe`. The next start deletes `calyx.old` (`remove_old_binary`).
//! The running process keeps its old image; the new version runs on restart.

use crate::update_check::parse_version;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

const DOWNLOAD_BASE: &str = "https://github.com/hector-horta/calyx/releases/download";
/// Release archives are ~10 MB; anything far bigger is not ours.
const MAX_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(120);
const BIN_NAME: &str = if cfg!(windows) { "calyx.exe" } else { "calyx" };
/// Lowercase path pieces of package-manager installs, which update themselves.
const MANAGED_PATHS: [&str; 6] = [
    ".cargo",
    "scoop",
    "winget",
    "cellar",
    "homebrew",
    "/nix/store",
];

/// Release asset for this platform, as named in release.yml.
pub fn asset_name() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some("calyx-windows-x86_64.zip"),
        ("linux", "x86_64") => Some("calyx-linux-x86_64.tar.gz"),
        ("macos", "aarch64") => Some("calyx-macos-arm64.tar.gz"),
        ("macos", "x86_64") => Some("calyx-macos-x86_64.tar.gz"),
        _ => None,
    }
}

/// Download URL of `asset` in release `tag`, rebuilt from the parsed version.
fn asset_url(tag: &str, asset: &str) -> Option<String> {
    let (major, minor, patch) = parse_version(tag)?;
    Some(format!("{DOWNLOAD_BASE}/v{major}.{minor}.{patch}/{asset}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `calyx.exe` → `calyx.old` (also `calyx` → `calyx.old`).
fn old_path(exe: &Path) -> PathBuf {
    exe.with_extension("old")
}

/// Whether `path` is the binary at the archive root (`calyx` or `./calyx`).
fn is_binary_entry(path: &Path) -> bool {
    let mut parts = path.components().filter(|c| *c != Component::CurDir);
    matches!(parts.next(), Some(Component::Normal(name)) if name == BIN_NAME)
        && parts.next().is_none()
}

/// Installed by cargo, scoop, winget, Homebrew or Nix.
fn is_managed(exe: &Path) -> bool {
    let path = exe.to_string_lossy().to_lowercase().replace('\\', "/");
    MANAGED_PATHS.iter().any(|piece| path.contains(piece))
}

/// Deletes the binary left by the last update (best effort, at startup).
pub fn remove_old_binary() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_path(&exe));
    }
}

/// Returns the SHA256 checksum of `asset` if found in `sums` (SHA256SUM format).
/// Otherwise returns `None`.
fn expected_sha256(sums: &str, asset: &str) -> Option<String> {
    for line in sums.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        };
        let hash = parts[0];
        let name = parts[1].trim_start_matches('*');
        if name == asset && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

/// The root `calyx.exe` of a release zip.
fn binary_from_zip(archive: &[u8]) -> Result<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(Cursor::new(archive)).context("not a zip archive")?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        if is_binary_entry(&path) {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }
    bail!("{BIN_NAME} not found in the zip")
}

/// The root `calyx` of a release tar.gz.
fn binary_from_tar_gz(archive: &[u8]) -> Result<Vec<u8>> {
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if entry.header().entry_type().is_file() && is_binary_entry(&path) {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }
    bail!("{BIN_NAME} not found in the tar.gz")
}

/// Writes `bytes` as `calyx.new`, moves the running binary to `calyx.old`
/// and the new one into its place; on failure the running binary is put back.
fn replace_exe(exe: &Path, bytes: &[u8]) -> Result<()> {
    let new = exe.with_extension("new");
    let old = old_path(exe);
    std::fs::write(&new, bytes).with_context(|| format!("cannot write {}", new.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755))?;
    }
    let _ = std::fs::remove_file(&old);
    if let Err(e) = std::fs::rename(exe, &old) {
        let _ = std::fs::remove_file(&new);
        return Err(e).context("cannot move the running binary aside");
    }
    if let Err(e) = std::fs::rename(&new, exe) {
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(&new);
        return Err(e).context("cannot put the new binary in place");
    }
    Ok(())
}

/// Downloads the content from `url` into a byte vector, respecting timeout and size limits.
async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let mut response = client.get(url).send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() > MAX_DOWNLOAD_BYTES {
            bail!("{url} is larger than expected");
        }
    }
    Ok(bytes)
}

/// Downloads and verifies a new release version, then replaces the current binary if valid.
pub async fn install(tag: &str) -> Result<()> {
    let Some(asset) = asset_name() else {
        bail!("no release build for this platform")
    };
    let Some(url) = asset_url(tag, asset) else {
        bail!("invalid release tag {tag}")
    };
    let exe = std::env::current_exe()?.canonicalize()?;
    if is_managed(&exe) {
        bail!("installed by a package manager; update calyx with it");
    }
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("calyx/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let sums = String::from_utf8(download(&client, &format!("{url}.sha256")).await?)?;
    let Some(expected) = expected_sha256(&sums, asset) else {
        bail!("no checksum published for {asset}")
    };
    let archive = download(&client, &url).await?;
    if sha256_hex(&archive) != expected {
        bail!("checksum mismatch for {asset}");
    }
    let binary = if asset.ends_with(".zip") {
        binary_from_zip(&archive)?
    } else {
        binary_from_tar_gz(&archive)?
    };
    replace_exe(&exe, &binary)
}

#[cfg(test)]
#[path = "self_update_tests.rs"]
mod tests;
