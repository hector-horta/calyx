//! Whether a newer calyx release exists on GitHub. Asked on every start with
//! a conditional request: the last answer and its ETag are cached next to
//! the catalog, and while nothing changed GitHub answers "304 Not Modified"
//! with no body. One request per start is far below GitHub's 60 per hour.
//! Offline, the last answer is used; with none, no update is known (calyx
//! never waits on this).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LATEST_URL: &str = "https://api.github.com/repos/hector-horta/calyx/releases/latest";
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

/// GitHub page of release `tag`. Rebuilt from the parsed numbers, so a tag
/// from the network can only ever produce this exact URL shape.
pub fn release_url(tag: &str) -> Option<String> {
    let (major, minor, patch) = parse_version(tag)?;
    Some(format!(
        "https://github.com/hector-horta/calyx/releases/tag/v{major}.{minor}.{patch}"
    ))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cache {
    /// Unix seconds of the last answer with a body.
    checked_at: u64,
    /// Tag of the latest release then, e.g. "v0.1.14".
    latest: String,
    /// GitHub's ETag for that answer, sent back as If-None-Match.
    #[serde(default)]
    etag: Option<String>,
}

/// GitHub's answer to the latest-release request.
#[derive(Debug, PartialEq)]
enum Answer {
    /// 304: the cached tag is still the latest.
    Unchanged,
    Latest {
        tag: String,
        etag: Option<String>,
    },
}

/// The latest tag known after asking, and the cache to write (if any).
/// No answer (offline, rate limited, bad JSON) keeps the last known tag.
fn resolve(
    cached: Option<Cache>,
    answer: Option<Answer>,
    now: u64,
) -> (Option<String>, Option<Cache>) {
    match answer {
        Some(Answer::Latest { tag, etag }) => {
            let cache = Cache {
                checked_at: now,
                latest: tag.clone(),
                etag,
            };
            (Some(tag), Some(cache))
        }
        Some(Answer::Unchanged) | None => (cached.map(|c| c.latest), None),
    }
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

/// Asks GitHub for the latest release, sending `etag` when there is one;
/// `None` on any network, HTTP or parse error.
async fn fetch_latest(etag: Option<&str>) -> Option<Answer> {
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("calyx/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let mut request = client
        .get(LATEST_URL)
        .header("Accept", "application/vnd.github+json");
    if let Some(etag) = etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }
    let response = request.send().await.ok()?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Some(Answer::Unchanged);
    }
    let response = response.error_for_status().ok()?;
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let release: Release = response.json().await.ok()?;
    parse_version(&release.tag_name)?;
    Some(Answer::Latest {
        tag: release.tag_name,
        etag,
    })
}

/// The latest release tag when it is newer than `current`, else `None`.
pub async fn newer_release(current: &str) -> Option<String> {
    let path = cache_file();
    let cached = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Cache>(&s).ok());
    let etag = cached.as_ref().and_then(|c| c.etag.clone());
    let answer = fetch_latest(etag.as_deref()).await;
    let (latest, to_save) = resolve(cached, answer, now_secs());
    if let Some(json) = to_save.and_then(|c| serde_json::to_string(&c).ok()) {
        let _ = std::fs::create_dir_all(crate::paths::data_dir());
        let _ = std::fs::write(&path, json);
    }
    let latest = latest?;
    is_newer(&latest, current).then_some(latest)
}

#[cfg(test)]
#[path = "update_check_tests.rs"]
mod tests;
