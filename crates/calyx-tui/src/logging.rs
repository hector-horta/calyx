//! Logs to `calyx.log` in calyx's data dir: the terminal belongs to the UI,
//! so nothing may be printed there. The level comes from `CALYX_LOG`
//! (`EnvFilter` syntax, e.g. `CALYX_LOG=debug`); by default calyx logs at
//! info and libraries at warn. The file is capped: at start, a log over
//! `MAX_BYTES` becomes `calyx.log.1`, replacing the previous one.

use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

/// Size at which the log is set aside at start.
const MAX_BYTES: u64 = 1024 * 1024;
/// calyx and calyx_core at info (the filter matches by prefix), the rest at warn.
const DEFAULT_FILTER: &str = "warn,calyx=info";

/// `calyx.log` in calyx's data dir.
pub fn log_file() -> PathBuf {
    calyx_core::paths::data_dir().join("calyx.log")
}

/// Sets a log over `max_bytes` aside as `<log>.1` (replacing an older one).
fn rotate(path: &Path, max_bytes: u64) {
    if let Ok(meta) = std::fs::metadata(path)
        && meta.len() > max_bytes
    {
        // Best effort: a log that cannot move just keeps growing.
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
}

/// Starts logging to `calyx.log` and returns its path; `None` on any
/// failure, and calyx then runs without a log.
pub fn init() -> Option<PathBuf> {
    let path = log_file();
    std::fs::create_dir_all(path.parent()?).ok()?;
    rotate(&path, MAX_BYTES);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    let filter =
        EnvFilter::try_from_env("CALYX_LOG").unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .try_init()
        .ok()?;
    Some(path)
}

#[cfg(test)]
#[path = "../tests/unit/logging.rs"]
mod tests;
