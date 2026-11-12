use crate::cdp::{CdpTarget, free_local_port, profile_devtools_port, send_cdp_command};
use crate::error::{CalyxError, CalyxResult};
use reqwest::Client;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::info;

pub fn find_browser_executable() -> CalyxResult<PathBuf> {
    match crate::browser_discovery::discover_browser() {
        Some(crate::browser_discovery::BrowserType::Chromium(p)) => Ok(p),
        Some(crate::browser_discovery::BrowserType::Firefox(p)) => Ok(p),
        Some(crate::browser_discovery::BrowserType::WebKit(p)) => Ok(p),
        None => Err(CalyxError::Auth(
            "No se encontró un navegador compatible. Presiona '?' en Calyx para ver la guía manual.".to_string(),
        )),
    }
}

/// Login browser profile (Apple ID cookies): in calyx's own data dir, not the
/// shared temp folder.
pub fn calyx_browser_profile_dir() -> PathBuf {
    crate::paths::browser_profile_dir()
}

pub struct LoginBrowserGuard {
    ws_url: String,
    child: Option<Child>,
}

impl LoginBrowserGuard {
    pub async fn kill_after_save(mut self) {
        let _ =
            send_cdp_command(&self.ws_url, r#"{"id":98,"method":"Storage.flushCookies"}"#).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        let _ = send_cdp_command(&self.ws_url, r#"{"id":99,"method":"Browser.close"}"#).await;
        tokio::time::sleep(Duration::from_millis(350)).await;
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for LoginBrowserGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

pub async fn interactive_apple_login(
    http: &Client,
    storefront: &str,
    dev_token: &str,
) -> CalyxResult<(Option<String>, String, LoginBrowserGuard)> {
    let browser = crate::browser_discovery::discover_browser().ok_or_else(|| {
        CalyxError::Auth("No se encontró un navegador compatible (Chromium o Firefox). Presiona '?' en Calyx para ver la guía manual.".to_string())
    })?;

    // Firefox and WebKit: no debugging port, the token is read from disk.
    let cookie_store: Option<(crate::login_cookies::TokenReader, &str)> = match &browser {
        crate::browser_discovery::BrowserType::Firefox(_) => Some((
            crate::login_firefox::extract_media_user_token_from_firefox_profiles,
            "Timeout esperando inicio de sesión en Firefox o fork compatible",
        )),
        crate::browser_discovery::BrowserType::WebKit(_) => Some((
            crate::login_webkit::extract_token_from_webkit,
            "Timeout en Safari/WebKit. Si macOS bloqueó permisos, usa '?' para modo manual.",
        )),
        crate::browser_discovery::BrowserType::Chromium(_) => None,
    };
    if let Some((read, timeout_msg)) = cookie_store {
        info!("Iniciando flujo de login por cookies del navegador...");
        let token =
            crate::login_cookies::wait_for_live_token(http, dev_token, read, timeout_msg).await?;
        let guard = LoginBrowserGuard {
            ws_url: String::new(),
            child: None,
        };
        return Ok((None, token, guard));
    }

    let browser_bin = match browser {
        crate::browser_discovery::BrowserType::Chromium(p) => p,
        _ => unreachable!(),
    };
    let profile_dir = calyx_browser_profile_dir();
    let _ = std::fs::create_dir_all(&profile_dir);

    // A window left over on *our* profile would keep the profile locked, so
    // close it. Only that browser: its port comes from our own profile dir.
    if let Some(old_port) = profile_devtools_port(&profile_dir) {
        let old_targets = format!("http://127.0.0.1:{old_port}/json");
        if let Ok(resp) = http.get(&old_targets).send().await
            && let Ok(targets) = resp.json::<Vec<CdpTarget>>().await
        {
            // The port file may be stale (another program could own the
            // port now): only close our own login window, on apple.com.
            let ours = targets.iter().filter(|t| {
                t.url
                    .as_deref()
                    .is_some_and(|u| u.starts_with("https://music.apple.com"))
            });
            for t in ours {
                if let Some(ws) = t.ws_url.as_deref() {
                    let _ = send_cdp_command(ws, r#"{"id":99,"method":"Browser.close"}"#).await;
                }
            }
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    }

    let port = free_local_port().ok_or_else(|| {
        CalyxError::Auth("No hay un puerto local libre para la ventana de login.".to_string())
    })?;
    let targets_url = format!("http://127.0.0.1:{port}/json");

    let sf = if storefront.len() == 2 {
        storefront
    } else {
        "us"
    };
    let login_url = format!("https://music.apple.com/{sf}/browse");
    info!(
        "Abriendo ventana de inicio de sesión Apple Music: {}",
        login_url
    );

    let mut child = spawn_app_window(&browser_bin, &profile_dir, &login_url, port)?;
    let mut cdp_was_online = false;
    let mut rejected: Option<String> = None;

    for i in 0..180 {
        tokio::time::sleep(Duration::from_millis(1000)).await;

        let Ok(resp) = http.get(&targets_url).send().await else {
            if cdp_was_online || (i > 6 && child.try_wait().ok().flatten().is_some()) {
                return Err(CalyxError::Auth(
                    "Ventana de inicio de sesión cerrada antes de completar el acceso.".to_string(),
                ));
            }
            continue;
        };
        cdp_was_online = true;
        let Ok(targets) = resp.json::<Vec<CdpTarget>>().await else {
            continue;
        };

        for target in targets {
            let is_page = target.target_type.as_deref() == Some("page");
            let url = target.url.as_deref().unwrap_or("");
            let is_main_music_origin = url.starts_with("https://music.apple.com");
            if is_page
                && is_main_music_origin
                && let Some(ws_url) = target.ws_url.as_deref()
            {
                let Some((opt_dev, token)) = probe_cdp_for_verified_token(ws_url).await else {
                    continue;
                };
                if rejected.as_deref() == Some(token.as_str()) {
                    continue;
                }
                let test_dev = opt_dev.as_deref().unwrap_or(dev_token);
                if !test_dev.is_empty() && !token_is_live(http, test_dev, &token).await {
                    info!("Token en perfil del navegador expirado; esperando login manual.");
                    rejected = Some(token);
                    continue;
                }
                let guard = LoginBrowserGuard {
                    ws_url: ws_url.to_string(),
                    child: Some(child),
                };
                return Ok((opt_dev, token, guard));
            }
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    Err(CalyxError::Auth(
        "Tiempo de espera agotado (3 min) esperando inicio de sesión en Apple Music.".to_string(),
    ))
}

/// Returns true only if Apple's API accepts the Media-User-Token (HTTP 2xx).
pub async fn token_is_live(http: &Client, dev_token: &str, user_token: &str) -> bool {
    http.get("https://amp-api.music.apple.com/v1/me/storefront")
        .header("Authorization", format!("Bearer {dev_token}"))
        .header("Media-User-Token", user_token)
        .header("Origin", "https://music.apple.com")
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn spawn_app_window(
    bin: &std::path::Path,
    profile_dir: &std::path::Path,
    url: &str,
    port: u16,
) -> CalyxResult<Child> {
    Command::new(bin)
        .arg(format!("--app={url}"))
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        // Loopback-only, per-login port. No --remote-allow-origins: our CDP
        // client sends no Origin header, so web pages cannot attach.
        .arg(format!("--remote-debugging-port={port}"))
        .arg("--window-size=520,700")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-extensions")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| CalyxError::Auth(format!("No se pudo lanzar ventana de login: {e}")))
}

async fn probe_cdp_for_verified_token(ws_url: &str) -> Option<(Option<String>, String)> {
    let (dev1, first_tok) = probe_cdp_for_tokens(ws_url).await?;
    let mk_ready_cmd = r#"{"id":4,"method":"Runtime.evaluate","params":{"expression":"(() => { try { const mk = window.MusicKit && window.MusicKit.getInstance && window.MusicKit.getInstance(); if (mk && mk.isAuthorized && mk.musicUserToken && mk.musicUserToken.length > 20) return 'READY'; for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i) || ''; if (k.includes('media-user-token') && (localStorage.getItem(k) || '').length > 20) return 'READY'; } return 'WAITING'; } catch(e) { return 'WAITING'; } })()","returnByValue":true}}"#;
    if !send_cdp_command(ws_url, mk_ready_cmd)
        .await?
        .contains("READY")
    {
        return None;
    }
    tokio::time::sleep(Duration::from_millis(800)).await;
    let (dev2, second_tok) = probe_cdp_for_tokens(ws_url).await?;
    (first_tok == second_tok).then_some((dev2.or(dev1), second_tok))
}

async fn probe_cdp_for_tokens(ws_url: &str) -> Option<(Option<String>, String)> {
    let js_cmd = r#"{"id":3,"method":"Runtime.evaluate","params":{"expression":"(() => { try { const mk = window.MusicKit && window.MusicKit.getInstance && window.MusicKit.getInstance(); let d = (mk && mk.developerToken) || ''; let u = (mk && mk.musicUserToken) || ''; for (let i = 0; i < localStorage.length; i++) { const k = (localStorage.key(i) || '').toLowerCase(); if (!d && (k.includes('developer-token') || k.includes('developertoken'))) d = localStorage.getItem(localStorage.key(i)) || ''; if (!u && k.includes('media-user-token')) u = localStorage.getItem(localStorage.key(i)) || ''; } return JSON.stringify({ dev: d.length > 50 ? d : null, user: u.length > 20 ? u : null }); } catch(e) { return ''; } })()","returnByValue":true}}"#;
    if let Some(raw) = send_cdp_command(ws_url, js_cmd).await
        && let Some(res) = extract_tokens_from_cdp_eval(&raw)
    {
        return Some(res);
    }
    for cmd in [
        // js_cmd already probed
        r#"{"id":1,"method":"Storage.getCookies"}"#,
        r#"{"id":2,"method":"Network.getAllCookies"}"#,
    ] {
        if let Some(raw) = send_cdp_command(ws_url, cmd).await
            && let Some(tok) = extract_token_from_cdp_json(&raw)
        {
            return Some((None, tok));
        }
    }
    None
}

pub fn extract_tokens_from_cdp_eval(raw_json: &str) -> Option<(Option<String>, String)> {
    let val: serde_json::Value = serde_json::from_str(raw_json).ok()?;
    let str_val = val
        .pointer("/result/result/value")
        .and_then(|v| v.as_str())?;
    let inner: serde_json::Value = serde_json::from_str(str_val).ok()?;
    let user = inner
        .get("user")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| s.len() > 20)?
        .to_string();
    let dev = inner
        .get("dev")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| s.len() > 50)
        .map(String::from);
    Some((dev, user))
}

pub fn extract_token_from_cdp_json(raw_json: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(raw_json).ok()?;
    if let Some(cookies) = val.pointer("/result/cookies").and_then(|c| c.as_array()) {
        for cookie in cookies {
            let name = cookie.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.eq_ignore_ascii_case("media-user-token")
                && let Some(tok) = cookie.get("value").and_then(|v| v.as_str())
            {
                let clean = tok.trim();
                if clean.len() > 20 {
                    return Some(clean.to_string());
                }
            }
        }
    }
    if let Some(eval_str) = val.pointer("/result/result/value").and_then(|v| v.as_str()) {
        let clean = eval_str.trim();
        if clean.len() > 20 {
            return Some(clean.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_cdp_json() {
        let cookies_json = r#"{"id":1,"result":{"cookies":[{"name":"itua","value":"cl"},{"name":"media-user-token","value":"Aq123456789012345678901234567890==","domain":".music.apple.com"}]}}"#;
        assert_eq!(
            extract_token_from_cdp_json(cookies_json).as_deref(),
            Some("Aq123456789012345678901234567890==")
        );

        let eval_json = r#"{"id":2,"result":{"result":{"type":"string","value":"MusicKitUserToken98765432109876543210"}}}"#;
        assert_eq!(
            extract_token_from_cdp_json(eval_json).as_deref(),
            Some("MusicKitUserToken98765432109876543210")
        );
    }

    #[test]
    fn test_extract_tokens_from_cdp_eval() {
        let raw = r#"{"id":3,"result":{"result":{"type":"string","value":"{\"dev\":\"eyJhbGciOiJFUzI1NiIsabcdefghijklmnopqrstuvwxyz01234567890123456789\",\"user\":\"MusicKitUserToken98765432109876543210\"}"}}}"#;
        let (dev, user) = extract_tokens_from_cdp_eval(raw).unwrap();
        assert!(dev.unwrap().starts_with("eyJ"));
        assert_eq!(user, "MusicKitUserToken98765432109876543210");
    }
}
