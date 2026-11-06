//! Writes to the user's Apple Music library (needs a logged-in session).

use crate::album_id::is_catalog_album_id;
use crate::error::{CalyxError, CalyxResult};
use reqwest::{Client, StatusCode};

const LIBRARY_URL: &str = "https://amp-api.music.apple.com/v1/me/library";

/// Target of `POST /v1/me/library?ids[albums]=<id>`; the id is validated
/// before it is put into the URL.
fn add_album_url(album_id: &str) -> CalyxResult<String> {
    if !is_catalog_album_id(album_id) {
        return Err(CalyxError::Validation(format!(
            "Invalid album id: {album_id:?}"
        )));
    }
    Ok(format!("{LIBRARY_URL}?ids[albums]={album_id}"))
}

/// Apple answers 202 Accepted, also when the album was already there.
fn check_status(status: StatusCode) -> CalyxResult<()> {
    if status.is_success() {
        return Ok(());
    }
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(CalyxError::Auth(format!(
            "Sesión Apple expirada (HTTP {status}). Presiona 'u' para login."
        )));
    }
    Err(CalyxError::Validation(format!(
        "Apple library HTTP {status}"
    )))
}

/// Adds a catalog album to the user's library.
pub(crate) async fn add_album_to_library(
    http: &Client,
    dev_token: &str,
    user_token: &str,
    album_id: &str,
) -> CalyxResult<()> {
    crate::library::require_session(dev_token, user_token, "agregar a tu biblioteca")?;
    let url = add_album_url(album_id)?;
    let resp = http
        .post(url)
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send()
        .await?;
    check_status(resp.status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_carries_a_valid_album_id() {
        assert_eq!(
            add_album_url("1475002920").unwrap(),
            "https://amp-api.music.apple.com/v1/me/library?ids[albums]=1475002920"
        );
    }

    #[test]
    fn url_rejects_ids_that_could_change_the_query() {
        assert!(add_album_url("1&ids[songs]=2").is_err());
        assert!(add_album_url("").is_err());
        assert!(add_album_url("l.abc").is_err());
    }

    #[test]
    fn status_mapping() {
        assert!(check_status(StatusCode::ACCEPTED).is_ok());
        assert!(matches!(
            check_status(StatusCode::UNAUTHORIZED),
            Err(CalyxError::Auth(_))
        ));
        assert!(matches!(
            check_status(StatusCode::FORBIDDEN),
            Err(CalyxError::Auth(_))
        ));
        assert!(matches!(
            check_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CalyxError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn needs_a_session_before_any_request() {
        let res = add_album_to_library(&Client::new(), "", "", "1475002920").await;
        assert!(matches!(res, Err(CalyxError::Auth(_))));
    }
}
