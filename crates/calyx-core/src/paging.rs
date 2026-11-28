//! Paginated GETs on the user-scoped Apple Music API (`/v1/me/...`).

use crate::error::{CalyxError, CalyxResult};
use crate::progress::LoadProgress;
use reqwest::Client;
use serde_json::Value;

const API_ROOT: &str = "https://amp-api.music.apple.com";
pub(crate) const PAGE_SIZE: usize = 100;

/// Items on one page and the path of the next one. Only relative `/v1/`
/// paths are followed, so a response can never redirect us to another host.
pub(crate) fn page_info(body: &str) -> CalyxResult<(usize, Option<String>)> {
    let val: Value = serde_json::from_str(body)
        .map_err(|e| CalyxError::Validation(format!("Invalid Apple Music page JSON: {e}")))?;
    let count = val
        .get("data")
        .and_then(|d| d.as_array())
        .map_or(0, |a| a.len());
    let next = val
        .get("next")
        .and_then(|n| n.as_str())
        .filter(|n| n.starts_with("/v1/"))
        .map(str::to_string);
    Ok((count, next))
}

/// Total items Apple reports for the whole list (`meta.total`), if any.
pub(crate) fn page_total(body: &str) -> Option<usize> {
    let val: Value = serde_json::from_str(body).ok()?;
    val.pointer("/meta/total")?.as_u64().map(|t| t as usize)
}

/// GETs `first_path` and every following page until `max_items` or the end,
/// counting items into `progress` page by page.
pub(crate) async fn get_paged(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    first_path: String,
    max_items: usize,
    what: &str,
    progress: Option<&LoadProgress>,
) -> CalyxResult<Vec<String>> {
    let mut bodies = Vec::new();
    let mut seen = 0;
    let mut next = Some(first_path);
    while let Some(path) = next.take() {
        let resp = http
            .get(format!("{API_ROOT}{path}"))
            .header("Authorization", format!("Bearer {dev_token}"))
            .header("Media-User-Token", user_token)
            .header("Origin", "https://music.apple.com")
            .header("Referer", "https://music.apple.com/")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(CalyxError::Auth(format!(
                "{what} HTTP status: {}",
                resp.status()
            )));
        }
        let body = resp.text().await?;
        let (count, next_path) = page_info(&body)?;
        seen += count;
        if let Some(p) = progress {
            if let Some(total) = page_total(&body) {
                p.set_total(total.min(max_items));
            }
            p.add(count.min(max_items.saturating_sub(seen - count)));
        }
        bodies.push(body);
        next = next_path.filter(|_| count > 0 && seen < max_items);
    }
    Ok(bodies)
}

#[cfg(test)]
#[path = "../tests/unit/paging.rs"]
mod tests;
