//! 'V' when the top bar announces a newer release: the first 'V' asks, a
//! second one within a few seconds downloads it and replaces the binary on
//! disk (`calyx_core::self_update`). The new version runs on the next start;
//! when the install fails, the release page opens instead.

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use crate::notice::Tone;
use calyx_core::Language;
use ratatui::style::{Color, Modifier, Style};
use std::time::{Duration, Instant};

/// A second 'V' within this time confirms (same as the 'd' confirmation).
const CONFIRM_WINDOW: Duration = Duration::from_secs(4);
/// How long the top bar says the install failed.
const FAILED_TTL: Duration = Duration::from_secs(6);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum UpdateState {
    #[default]
    Idle,
    /// First 'V' pressed at this moment.
    Asked(Instant),
    /// Downloading since this moment (drives the spinner).
    Installing(Instant),
    /// On disk; runs after a restart.
    Installed,
    /// The install failed at this moment; the release page was opened.
    Failed(Instant),
}

/// What the top bar shows after "-> vX" for `update`: text, style, and
/// whether the [V] key goes after it. Every step of the update is shown
/// there, next to the announcement the user is looking at.
pub fn badge(lang: Language, update: UpdateState) -> (String, Style, bool) {
    let es = lang == Language::Es;
    let plain = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    match update {
        UpdateState::Asked(at) if at.elapsed() < CONFIRM_WINDOW => {
            let left_ms = CONFIRM_WINDOW.saturating_sub(at.elapsed()).as_millis();
            let left = left_ms.div_ceil(1000);
            let text = if es {
                format!("presiona V otra vez para instalar ({left}s)")
            } else {
                format!("press V again to install ({left}s)")
            };
            (text, Tone::Action.style(), false)
        }
        UpdateState::Installing(since) => {
            let spin = crate::spinner::frame_since(since);
            let word = if es {
                "descargando…"
            } else {
                "downloading…"
            };
            (format!("{spin} {word}"), plain, false)
        }
        UpdateState::Installed => {
            let text = if es {
                "instalada: reinicia calyx"
            } else {
                "installed: restart calyx"
            };
            (text.to_string(), Tone::Result.style(), false)
        }
        UpdateState::Failed(at) if at.elapsed() < FAILED_TTL => {
            let text = if es {
                "no se pudo instalar: abriendo la página"
            } else {
                "could not install: opening the page"
            };
            (text.to_string(), Tone::Error.style(), false)
        }
        _ => {
            let text = if es { "disponible" } else { "available" };
            (text.to_string(), plain, true)
        }
    }
}

impl TuiApp {
    /// Handles the 'V' key for updating Calyx to a newer version.
    pub fn update_key(&mut self) {
        let Some(tag) = crate::version_badge::newer() else {
            return;
        };
        if calyx_core::self_update::asset_name().is_none() {
            crate::version_badge::open_release(self);
            return;
        }

        match self.update {
            UpdateState::Installing(_) | UpdateState::Installed => {}
            UpdateState::Asked(at) if at.elapsed() < CONFIRM_WINDOW => self.start_install(tag),
            _ => self.update = UpdateState::Asked(Instant::now()),
        };
    }

    /// Starts the installation process for the given version tag.
    fn start_install(&mut self, tag: &'static str) {
        self.update = UpdateState::Installing(Instant::now());
        self.spawn_event(async move {
            let res = calyx_core::self_update::install(tag)
                .await
                .map_err(|e| format!("{e:#}"));
            BgEvent::UpdateInstalled(tag, res)
        });
    }

    /// The install finished: ask for a restart, or fall back to the page.
    pub(crate) fn on_update_installed(&mut self, tag: &str, res: Result<(), String>) {
        match res {
            Ok(()) => self.update = UpdateState::Installed,
            Err(err) => {
                tracing::warn!(%tag, error = %err, "self-update failed");
                self.update = UpdateState::Failed(Instant::now());
                crate::version_badge::open_release(self);
            }
        }
    }
}
