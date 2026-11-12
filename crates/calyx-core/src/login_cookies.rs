//! Login through a browser's cookie store (Firefox and WebKit families):
//! the page opens in the browser and calyx waits for a live token on disk.

use crate::error::{CalyxError, CalyxResult};
use reqwest::Client;
use std::time::{Duration, Instant};

/// Reads the media-user-token from a browser's cookie store, if present.
pub(crate) type TokenReader = fn() -> Option<String>;

const TIMEOUT: Duration = Duration::from_secs(180);

/// Opens the login page and polls `read` until it yields a token Apple
/// accepts. The cookie of an expired session is checked once and skipped.
pub(crate) async fn wait_for_live_token(
    http: &Client,
    dev_token: &str,
    read: TokenReader,
    timeout_msg: &str,
) -> CalyxResult<String> {
    crate::browser_choice::open_login_page("https://music.apple.com")?;
    let start = Instant::now();
    let mut rejected: Option<String> = None;
    while start.elapsed() < TIMEOUT {
        if let Some(token) = read().filter(|t| rejected.as_ref() != Some(t)) {
            if dev_token.is_empty() || crate::login::token_is_live(http, dev_token, &token).await {
                return Ok(token);
            }
            rejected = Some(token);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(CalyxError::Auth(timeout_msg.to_string()))
}
