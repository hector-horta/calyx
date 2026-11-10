//! Whether a newer calyx release exists on GitHub. Asked at most once a day:
//! the answer is cached next to the catalog, and any network or parse error
//! just means "no update known" (calyx never waits on this).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LATEST_URL: &str = "https://api.github.com/repos/hector-horta/calyx/releases/latest";
const DAY_SECS: u64 = 24 * 60 * 60;
const TIMEOUT: Duration = Duration::from_secs(5);

/// "v0.1.14" or "0.1.14" as (major, minor, patch); `None` if it is not that.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.trim().trim_start_matches('v').split('.');
    let mut next = || parts.next()?.parse::<u64>().ok();
    let version = (next()?, next()?, next()?);
    parts.next().is_none().then_some(version)
}

/// True when `latest` is a valid version above `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cache {
    /// Unix seconds of the last successful check.
    checked_at: u64,
    /// Tag of the latest release then, e.g. "v0.1.14".
    latest: String,
}

/// The cached tag when it was checked less than a day before `now`.
fn fresh(cache: &Cache, now: u64) -> Option<String> {
    (now.saturating_sub(cache.checked_at) < DAY_SECS).then(|| cache.latest.clone())
}

fn cache_file() -> PathBuf {
    crate::paths::data_dir().join("update_check.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

/// Tag of the latest GitHub release, validated as a version.
async fn fetch_latest() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("calyx/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let release: Release = client
        .get(LATEST_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    parse_version(&release.tag_name).map(|_| release.tag_name)
}

/// The latest release tag when it is newer than `current`, else `None`.
/// Uses the day-old cache when there is one; otherwise asks GitHub and
/// caches the answer (a failed request is not cached, so it is retried on
/// the next start).
pub async fn newer_release(current: &str) -> Option<String> {
    let path = cache_file();
    let now = now_secs();
    let cached = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Cache>(&s).ok())
        .and_then(|c| fresh(&c, now));
    let latest = match cached {
        Some(tag) => tag,
        None => {
            let tag = fetch_latest().await?;
            let cache = Cache {
                checked_at: now,
                latest: tag.clone(),
            };
            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = std::fs::create_dir_all(crate::paths::data_dir());
                let _ = std::fs::write(&path, json);
            }
            tag
        }
    };
    is_newer(&latest, current).then_some(latest)
}

#[cfg(test)]
#[path = "update_check_tests.rs"]
mod tests;
