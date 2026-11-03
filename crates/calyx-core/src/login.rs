use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use reqwest::Client;
use tracing::info;
use crate::cdp::{send_cdp_command, CdpTarget, CDP_PORT};
use crate::error::{CalyxError, CalyxResult};

pub fn find_browser_executable() -> CalyxResult<PathBuf> {
    if let Ok(bin) = std::env::var("CALYX_BROWSER_BIN").or_else(|_| std::env::var("BROWSER")) {
        let p = PathBuf::from(bin);
        if p.exists() {
            return Ok(p);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();
        for base in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
            if let Ok(dir) = std::env::var(base) {
                let pb = PathBuf::from(dir);
                candidates.push(pb.join(r"Microsoft\Edge\Application\msedge.exe"));
                candidates.push(pb.join(r"Google\Chrome\Application\chrome.exe"));
                candidates.push(pb.join(r"BraveSoftware\Brave-Browser\Application\brave.exe"));
            }
        }
        candidates.into_iter().find(|p| p.exists()).ok_or_else(|| {
            CalyxError::Auth("No se encontró un navegador compatible. Consulta 'Autenticación' en README.".to_string())
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(bin) = std::env::var("CALYX_BROWSER_BIN").or_else(|_| std::env::var("BROWSER")) {
            let p = PathBuf::from(bin);
            if p.exists() { return Ok(p); }
        }
        let list = [
            "/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/chromium",
            "/usr/bin/chromium-browser", "/usr/bin/brave-browser", "/usr/bin/microsoft-edge",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ];
        list.into_iter().map(PathBuf::from).find(|p| p.exists())
            .ok_or_else(|| CalyxError::Auth("No se encontró un navegador compatible. Consulta 'Autenticación' en README.".to_string()))
    }
}

pub fn calyx_browser_profile_dir() -> PathBuf {
    std::env::temp_dir().join("calyx_apple_auth_profile")
}

pub struct LoginBrowserGuard {
    ws_url: String,
    child: Option<Child>,
}

impl LoginBrowserGuard {
    pub async fn kill_after_save(mut self) {
        let _ = send_cdp_command(&self.ws_url, r#"{"id":98,"method":"Storage.flushCookies"}"#).await;
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
) -> CalyxResult<(String, LoginBrowserGuard)> {
    let browser_bin = find_browser_executable()?;
    let profile_dir = calyx_browser_profile_dir();
    let _ = std::fs::create_dir_all(&profile_dir);

    let targets_url = format!("http://127.0.0.1:{CDP_PORT}/json");
    if let Ok(resp) = http.get(&targets_url).send().await {
        if let Ok(targets) = resp.json::<Vec<CdpTarget>>().await {
            for t in targets {
                if let Some(ws) = t.ws_url.as_deref() {
                    let _ = send_cdp_command(ws, r#"{"id":99,"method":"Browser.close"}"#).await;
                }
            }
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    }

    let sf = if storefront.len() == 2 { storefront } else { "us" };
    let login_url = format!("https://music.apple.com/{sf}/browse");
    info!("Abriendo ventana de inicio de sesión Apple Music: {}", login_url);

    let mut child = spawn_app_window(&browser_bin, &profile_dir, &login_url)?;
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
            if is_page && is_main_music_origin {
                if let Some(ws_url) = target.ws_url.as_deref() {
                    let Some(token) = probe_cdp_for_verified_token(ws_url).await else { continue };
                    if rejected.as_deref() == Some(token.as_str()) {
                        continue;
                    }
                    if !token_is_live(http, dev_token, &token).await {
                        info!("Token en perfil del navegador expirado; esperando login manual.");
                        rejected = Some(token);
                        continue;
                    }
                    let guard = LoginBrowserGuard { ws_url: ws_url.to_string(), child: Some(child) };
                    return Ok((token, guard));
                }
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

fn spawn_app_window(bin: &PathBuf, profile_dir: &PathBuf, url: &str) -> CalyxResult<Child> {
    Command::new(bin)
        .arg(format!("--app={url}"))
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg(format!("--remote-debugging-port={CDP_PORT}"))
        .arg("--remote-allow-origins=*")
        .arg("--window-size=520,700")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-extensions")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| CalyxError::Auth(format!("No se pudo lanzar ventana de login: {e}")))
}

async fn probe_cdp_for_verified_token(ws_url: &str) -> Option<String> {
    let first_tok = probe_cdp_for_media_token(ws_url).await?;
    let mk_ready_cmd = r#"{"id":4,"method":"Runtime.evaluate","params":{"expression":"(() => { try { const mk = window.MusicKit && window.MusicKit.getInstance && window.MusicKit.getInstance(); if (mk && mk.isAuthorized && mk.musicUserToken && mk.musicUserToken.length > 20) return 'READY'; for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i) || ''; if (k.includes('media-user-token') && (localStorage.getItem(k) || '').length > 20) return 'READY'; } return 'WAITING'; } catch(e) { return 'WAITING'; } })()","returnByValue":true}}"#;
    if !send_cdp_command(ws_url, mk_ready_cmd).await?.contains("READY") {
        return None;
    }
    tokio::time::sleep(Duration::from_millis(800)).await;
    let second_tok = probe_cdp_for_media_token(ws_url).await?;
    (first_tok == second_tok).then_some(second_tok)
}

async fn probe_cdp_for_media_token(ws_url: &str) -> Option<String> {
    let js_cmd = r#"{"id":3,"method":"Runtime.evaluate","params":{"expression":"(() => { try { const mk = window.MusicKit && window.MusicKit.getInstance && window.MusicKit.getInstance(); if (mk && mk.musicUserToken && mk.musicUserToken.length > 20) return mk.musicUserToken; for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i) || ''; if (k.includes('media-user-token')) { const v = localStorage.getItem(k) || ''; if (v.length > 20) return v; } } return ''; } catch(e) { return ''; } })()","returnByValue":true}}"#;
    for cmd in [
        js_cmd,
        r#"{"id":1,"method":"Storage.getCookies"}"#,
        r#"{"id":2,"method":"Network.getAllCookies"}"#,
    ] {
        if let Some(raw) = send_cdp_command(ws_url, cmd).await {
            if let Some(tok) = extract_token_from_cdp_json(&raw) {
                return Some(tok);
            }
        }
    }
    None
}


pub fn extract_token_from_cdp_json(raw_json: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(raw_json).ok()?;
    if let Some(cookies) = val.pointer("/result/cookies").and_then(|c| c.as_array()) {
        for cookie in cookies {
            let name = cookie.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.eq_ignore_ascii_case("media-user-token") {
                if let Some(tok) = cookie.get("value").and_then(|v| v.as_str()) {
                    let clean = tok.trim();
                    if clean.len() > 20 {
                        return Some(clean.to_string());
                    }
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
}
