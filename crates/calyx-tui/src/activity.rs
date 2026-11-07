//! What the app is waiting for. Drives the spinner before the status message
//! and the "300/1000" counter after it while a paginated list loads.

use calyx_core::LoadProgress;
use std::sync::Arc;
use std::time::Instant;

/// Kinds of background work. Starting a kind that is already running keeps
/// one entry, so a replaced task (new song, new search) never leaks a spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    /// Search, sections, library, playlist or album shown in the track list.
    List,
    /// The user's playlists (view 5 or the 'm' picker).
    Playlists,
    /// Audio of the song that is about to play.
    Audio,
    /// Writes: add album to library, add song to playlist.
    Write,
}

impl crate::bg_events::BgEvent {
    /// The job this result completes. Audio is handled where the current song
    /// is known, since results of a replaced song must not stop the spinner.
    pub fn finished_job(&self) -> Option<Job> {
        use crate::bg_events::BgEvent as E;
        match self {
            E::TracksReady { .. } | E::PlaylistOpened(..) | E::PlaylistReady(..) => Some(Job::List),
            E::AlbumOpened(..) => Some(Job::List),
            E::PlaylistsReady(..) => Some(Job::Playlists),
            E::AlbumAddedToLibrary(..)
            | E::AddedToPlaylist { .. }
            | E::PlaylistCreated { .. }
            | E::RemovedFromPlaylist { .. } => Some(Job::Write),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Activity {
    jobs: Vec<Job>,
    since: Option<Instant>,
    progress: Option<Arc<LoadProgress>>,
}

impl Activity {
    pub fn start(&mut self, job: Job) {
        if self.jobs.is_empty() {
            self.since = Some(Instant::now());
        }
        if !self.jobs.contains(&job) {
            self.jobs.push(job);
        }
        if job == Job::List {
            self.progress = None;
        }
    }

    /// Starts a list load that reports progress; returns the counter to fill.
    pub fn start_counted(&mut self) -> Arc<LoadProgress> {
        self.start(Job::List);
        let progress = Arc::new(LoadProgress::default());
        self.progress = Some(progress.clone());
        progress
    }

    pub fn finish(&mut self, job: Job) {
        self.jobs.retain(|j| *j != job);
        if job == Job::List {
            self.progress = None;
        }
        if self.jobs.is_empty() {
            self.since = None;
        }
    }

    /// Spinner frame while anything is running.
    pub fn spinner(&self) -> Option<&'static str> {
        self.since.map(crate::spinner::frame_since)
    }

    /// "300/1000" (or "300" until the total is known) while a list loads page by page.
    pub fn counter(&self) -> Option<String> {
        let (loaded, total) = self.progress.as_ref()?.snapshot();
        match (loaded, total) {
            (0, _) => None,
            (n, Some(t)) => Some(format!("{n}/{t}")),
            (n, None) => Some(n.to_string()),
        }
    }

    /// Status line text: spinner, message, counter.
    pub fn decorate(&self, status: &str) -> String {
        match (self.spinner(), self.counter()) {
            (None, _) => status.to_string(),
            (Some(s), None) => format!("{s} {status}"),
            (Some(s), Some(c)) => format!("{s} {status} {c}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_runs_until_every_job_finishes() {
        let mut a = Activity::default();
        assert_eq!(a.decorate("Listo"), "Listo");
        a.start(Job::Audio);
        a.start(Job::Write);
        a.start(Job::Audio); // replaced song: still one Audio job
        a.finish(Job::Audio);
        assert!(a.spinner().is_some());
        a.finish(Job::Write);
        assert!(a.spinner().is_none());
    }

    #[test]
    fn counter_follows_the_shared_progress() {
        let mut a = Activity::default();
        let p = a.start_counted();
        assert!(a.counter().is_none());
        p.add(300);
        assert_eq!(a.counter().as_deref(), Some("300"));
        p.set_total(1000);
        let text = a.decorate("Cargando Biblioteca...");
        assert!(text.ends_with(" Cargando Biblioteca... 300/1000"), "{text}");
        a.finish(Job::List);
        assert_eq!(
            a.decorate("Biblioteca: 1000 pistas."),
            "Biblioteca: 1000 pistas."
        );
    }
}
