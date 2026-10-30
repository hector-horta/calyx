//! Minimal Chrome DevTools Protocol (CDP) WebSocket transport.

use std::time::Duration;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(crate) const CDP_PORT: u16 = 19222;

#[derive(Debug, Deserialize)]
pub(crate) struct CdpTarget {
    #[serde(rename = "type")]
    pub target_type: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub ws_url: Option<String>,
}

pub(crate) async fn send_cdp_command(ws_url: &str, json_cmd: &str) -> Option<String> {
    tokio::time::timeout(Duration::from_secs(10), async {
        let path = ws_url.strip_prefix(&format!("ws://127.0.0.1:{CDP_PORT}"))?;
        for _ in 0..6 {
            let Ok(mut stream) = TcpStream::connect(("127.0.0.1", CDP_PORT)).await else {
                tokio::time::sleep(Duration::from_millis(60)).await;
                continue;
            };
            let handshake = format!(
                "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{CDP_PORT}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
            );
            if stream.write_all(handshake.as_bytes()).await.is_err() {
                continue;
            }

            let mut hdr_buf = [0u8; 1024];
            let Ok(n) = stream.read(&mut hdr_buf).await else {
                continue;
            };
            if !String::from_utf8_lossy(&hdr_buf[..n]).contains("101") {
                tokio::time::sleep(Duration::from_millis(80)).await;
                continue;
            }

            let frame = encode_ws_client_frame(json_cmd.as_bytes());
            if stream.write_all(&frame).await.is_err() {
                return None;
            }
            let out = read_ws_text_frame(&mut stream).await;
            let _ = stream.write_all(&[0x88, 0x80, 0x00, 0x00, 0x00, 0x00]).await;
            let _ = stream.shutdown().await;
            return out;
        }
        None
    })
    .await
    .ok()
    .flatten()
}

fn encode_ws_client_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 14);
    frame.push(0x81);
    let len = payload.len();
    if len <= 125 {
        frame.push(0x80 | (len as u8));
    } else {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    }
    let mask = [0x37u8, 0xfa, 0x21, 0x3d];
    frame.extend_from_slice(&mask);
    for (i, &b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }
    frame
}

async fn read_ws_text_frame(stream: &mut TcpStream) -> Option<String> {
    let mut head = [0u8; 2];
    stream.read_exact(&mut head).await.ok()?;
    let mut len = (head[1] & 0x7F) as usize;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).await.ok()?;
        len = u16::from_be_bytes(ext) as usize;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).await.ok()?;
        len = u64::from_be_bytes(ext) as usize;
    }
    if len > 2 * 1024 * 1024 {
        return None;
    }
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await.ok()?;
    String::from_utf8(data).ok()
}
