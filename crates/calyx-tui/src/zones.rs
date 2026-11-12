//! Messages are shown where the eye already is: list work (loading,
//! syncing playlists, enqueue, remove...) in the list frame's bottom border,
//! playback in the player box. The self-update has its own place in the top
//! bar (version_badge.rs). Each zone keeps its own notice and progress line,
//! so a list message never hides a playback one.

use crate::activity::{Activity, Job};
use crate::notice::Notice;
use crate::ui::truncate;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::time::{Duration, Instant};

/// A finished list message (e.g. "Playlists synced: 12") stays this long.
const STATUS_TTL: Duration = Duration::from_secs(4);
/// Background work whose spinner belongs to the list zone.
const LIST_JOBS: [Job; 3] = [Job::List, Job::Playlists, Job::Write];
const PROGRESS: Style = Style::new().fg(Color::Gray).add_modifier(Modifier::ITALIC);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    List,
    Player,
}

/// One zone's messages: an action/error/result notice, or routine progress.
#[derive(Default)]
pub struct Slot {
    pub notice: Option<Notice>,
    pub status: String,
    status_at: Option<Instant>,
}

impl Slot {
    pub fn set_status(&mut self, text: String) {
        self.status = text;
        self.status_at = Some(Instant::now());
    }

    pub fn clear_status(&mut self) {
        self.status.clear();
        self.status_at = None;
    }

    fn live_notice(&self) -> Option<&Notice> {
        self.notice.as_ref().filter(|n| n.is_live())
    }
}

#[derive(Default)]
pub struct Zones {
    pub list: Slot,
    pub player: Slot,
}

impl Zones {
    pub fn get_mut(&mut self, zone: Zone) -> &mut Slot {
        match zone {
            Zone::List => &mut self.list,
            Zone::Player => &mut self.player,
        }
    }
}

/// The list frame's bottom border while the list zone has something to
/// say (a notice, work in progress, or a result from the last seconds);
/// `None` leaves the key hints there.
pub fn list_line(slot: &Slot, activity: &Activity, width: usize) -> Option<Line<'static>> {
    if let Some(notice) = slot.live_notice() {
        return Some(notice.line(width));
    }
    let busy = activity.busy(&LIST_JOBS);
    let fresh = slot.status_at.is_some_and(|at| at.elapsed() < STATUS_TTL);
    if slot.status.is_empty() || !(busy || fresh) {
        return None;
    }
    let text = activity.decorate_jobs(&LIST_JOBS, &slot.status);
    let text = truncate(&text, width.saturating_sub(3));
    Some(Line::from(Span::styled(format!(" {text} "), PROGRESS)))
}

#[cfg(test)]
#[path = "zones_tests.rs"]
mod tests;
