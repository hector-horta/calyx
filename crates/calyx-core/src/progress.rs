//! Progress of a paginated load, shared between the loading task and the UI.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Items loaded so far and, when the service reports it, how many there are.
/// Cheap to read every frame; written once per page.
#[derive(Debug, Default)]
pub struct LoadProgress {
    loaded: AtomicUsize,
    total: AtomicUsize,
}

impl LoadProgress {
    /// (loaded, total); total is `None` until the service reports it.
    pub fn snapshot(&self) -> (usize, Option<usize>) {
        let total = self.total.load(Ordering::Relaxed);
        (
            self.loaded.load(Ordering::Relaxed),
            (total > 0).then_some(total),
        )
    }

    pub fn add(&self, items: usize) {
        self.loaded.fetch_add(items, Ordering::Relaxed);
    }

    pub fn set_total(&self, total: usize) {
        self.total.store(total, Ordering::Relaxed);
    }
}

#[cfg(test)]
#[path = "../tests/unit/progress.rs"]
mod tests;
