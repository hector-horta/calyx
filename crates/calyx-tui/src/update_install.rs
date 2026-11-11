//! 'V' when the top bar announces a newer release: the first 'V' asks, a
//! second one within a few seconds downloads it and replaces the binary on
//! disk (`calyx_core::self_update`). The new version runs on the next start;
//! when the install fails, the release page opens instead.

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use crate::notice::{Hold, Notice, Tone};
use calyx_core::Language;
use std::time::{Duration, Instant};

/// A second 'V' within this time confirms (same as the 'd' confirmation).
const CONFIRM_WINDOW: Duration = Duration::from_secs(4);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum UpdateState {
    #[default]
    Idle,
    /// First 'V' pressed at this moment.
    Asked(Instant),
    Installing,
    /// On disk; runs after a restart.
    Installed,
}

enum Step<'a> {
    Ask,
    Installing,
    Installed,
    Failed(&'a str),
}

/// Text of each step for release `tag`, in `lang`.
fn text(lang: Language, step: Step, tag: &str) -> String {
    match (lang, step) {
        (Language::Es, Step::Ask) => format!("Presiona V otra vez para instalar {tag}"),
        (Language::En, Step::Ask) => format!("Press V again to install {tag}"),
        (Language::Es, Step::Installing) => format!("Descargando {tag}…"),
        (Language::En, Step::Installing) => format!("Downloading {tag}…"),
        (Language::Es, Step::Installed) => format!("{tag} instalada: reinicia calyx para usarla"),
        (Language::En, Step::Installed) => format!("{tag} installed: restart calyx to use it"),
        (Language::Es, Step::Failed(err)) => {
            format!("No se pudo actualizar ({err}); abriendo la página")
        }
        (Language::En, Step::Failed(err)) => format!("Update failed ({err}); opening the page"),
    }
}

impl TuiApp {
    fn update_notice(&mut self, tone: Tone, hold: Hold, step: Step, tag: &str) {
        let text = text(self.language, step, tag);
        self.notice = Some(Notice::held(text, tone, hold));
    }

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
            UpdateState::Installing => {}
            UpdateState::Installed => {
                self.update_notice(Tone::Result, Hold::UntilKey, Step::Installed, tag)
            }
            UpdateState::Asked(at) if at.elapsed() < CONFIRM_WINDOW => self.start_install(tag),
            _ => {
                self.update = UpdateState::Asked(Instant::now());
                self.update_notice(Tone::Action, Hold::Timed, Step::Ask, tag);
            }
        };
    }

    /// Starts the installation process for the given version tag.
    fn start_install(&mut self, tag: &'static str) {
        self.update = UpdateState::Installing;
        self.update_notice(Tone::Action, Hold::Pinned, Step::Installing, tag);
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
            Ok(()) => {
                self.update = UpdateState::Installed;
                self.update_notice(Tone::Result, Hold::UntilKey, Step::Installed, tag);
            }
            Err(err) => {
                self.update = UpdateState::Idle;
                self.update_notice(Tone::Error, Hold::Timed, Step::Failed(&err), tag);
                crate::version_badge::open_release(self);
            }
        }
    }
}
