//! Apple Music login from the UI: 'u', or at startup with `calyx -u <browser>`
//! (see `calyx_core::browser_choice`).

use crate::app::TuiApp;
use crate::bg_events::BgEvent;
use calyx_core::StreamingBackend;
use calyx_core::browser_discovery::{BrowserType, discover_browser};

impl TuiApp {
    /// 'u' or `-u <browser>`: warns about Safari's prompt, then logs in.
    pub fn begin_login(&mut self) {
        if self.login_in_progress {
            return;
        }
        let browser = discover_browser();
        let safari = matches!(browser, Some(BrowserType::WebKit(_)));
        self.status_message = if safari {
            self.language.msg_login_safari_alert().to_string()
        } else {
            self.language.msg_login_opening().to_string()
        };
        self.start_login();
    }

    pub fn start_login(&mut self) {
        if self.login_in_progress {
            return;
        }
        self.login_in_progress = true;
        let backend = self.backend.clone();
        self.spawn_event(async move {
            BgEvent::LoginDone(backend.login().await.map_err(|e| e.to_string()))
        });
    }
}
