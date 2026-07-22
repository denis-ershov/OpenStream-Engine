//! Playlist Edge: клиент запрашивает чистый m3u8 с роутера (без CA).
//! Сегменты остаются на CDN (absolute URL в media playlist).

use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use hyper::{Request, Response, StatusCode};
use ose_observe::{EngineEvent, EventKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

use crate::{full_body, parse_status, process_manifest_if_needed, read_http_headers, ProxyState, RespBody};

/// Twitch web Client-ID (публичный, как у embed/player).
const TWITCH_CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

const GQL_QUERY: &str = r#"query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $vodID: ID!, $isVod: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
    __typename
  }
  videoPlaybackAccessToken(id: $vodID, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isVod) {
    value
    signature
    __typename
  }
}"#;

/// `GET /twitch/<channel>` → clean master m3u8 (strip + master rewrite на Edge).
pub async fn handle_twitch_edge(
    state: Arc<ProxyState>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<RespBody>> {
    if !state.config.read().mode.allows_playlist_proxy() {
        return Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(full_body("edge unavailable in this mode"))
            .expect("response"));
    }
    if !state.config.read().twitch.enabled {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full_body("twitch plugin disabled"))
            .expect("response"));
    }

    let path = req.uri().path();
    let channel = path
        .strip_prefix("/twitch/")
        .unwrap_or("")
        .trim_matches('/')
        .split('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if channel.is_empty()
        || !channel
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full_body("usage: GET /twitch/<channel>"))
            .expect("response"));
    }

    let proxy_base = resolve_edge_proxy_base(&state, &req);
    if proxy_base.is_none() {
        tracing::warn!(
            "edge: proxy_public_url не задан и Host непригоден — master без rewrite; \
             media уйдут на CDN напрямую, реклама не strip'ится. \
             Задайте LuCI «Public Edge URL» = http://LAN_IP:18080"
        );
    } else {
        debug!(?proxy_base, "edge: master rewrite base");
    }

    info!(%channel, "edge: resolving twitch playlist");
    let (token, sig) = fetch_playback_token(&channel).await?;
    let usher_path = format!(
        "/api/channel/hls/{channel}.m3u8?client_id={TWITCH_CLIENT_ID}&token={}&sig={}&allow_source=true&allow_audio_only=true&fast_bread=true&playlist_include_framerate=true&reassignments_supported=true",
        urlencoding_minimal(&token),
        urlencoding_minimal(&sig)
    );
    let body = https_request(
        "usher.ttvnw.net",
        &usher_path,
        "GET",
        None,
        &[("Client-ID", TWITCH_CLIENT_ID)],
    )
    .await
    .context("usher fetch")?;

    if !body.windows(7).any(|w| w == b"#EXTM3U") {
        let preview = String::from_utf8_lossy(&body[..body.len().min(200)]);
        bail!("usher did not return m3u8: {preview}");
    }

    let url = format!("https://usher.ttvnw.net{usher_path}");
    let out = process_manifest_if_needed(
        &state,
        "usher.ttvnw.net",
        &usher_path,
        &url,
        &body,
        None,
        proxy_base,
    )
    .await?;

    state.status.push_event(EngineEvent::now(
        EventKind::ManifestProcessed,
        "twitch-edge",
        "usher.ttvnw.net",
        &usher_path,
        &format!("edge channel={channel}"),
        0,
    ));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/vnd.apple.mpegurl")
        .header("cache-control", "no-store")
        .header("access-control-allow-origin", "*")
        .header("content-length", out.len().to_string())
        .body(full_body(out))
        .expect("response"))
}

/// База для rewrite master → nested `/https://…`.
/// Приоритет: `proxy_public_url` → `http://{Host}` (не loopback).
fn resolve_edge_proxy_base(state: &ProxyState, req: &Request<hyper::body::Incoming>) -> Option<String> {
    if let Some(u) = state.config.read().proxy_public_url.clone() {
        let t = u.trim().trim_end_matches('/');
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let host = req
        .headers()
        .get(hyper::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|h| !h.is_empty())?;
    let lower = host.to_ascii_lowercase();
    let loopback = lower.starts_with("127.")
        || lower == "localhost"
        || lower.starts_with("localhost:")
        || lower == "[::1]"
        || lower.starts_with("[::1]:");
    if loopback {
        return None;
    }
    Some(format!("http://{host}"))
}

async fn fetch_playback_token(channel: &str) -> Result<(String, String)> {
    let variables = serde_json::json!({
        "isLive": true,
        "login": channel,
        "isVod": false,
        "vodID": "",
        "playerType": "site"
    });
    let payload = serde_json::json!({
        "operationName": "PlaybackAccessToken_Template",
        "query": GQL_QUERY,
        "variables": variables
    });
    let body = payload.to_string();
    let resp = https_request(
        "gql.twitch.tv",
        "/gql",
        "POST",
        Some(body.as_bytes()),
        &[
            ("Client-ID", TWITCH_CLIENT_ID),
            ("Content-Type", "application/json"),
        ],
    )
    .await
    .context("gql token")?;

    let v: serde_json::Value = serde_json::from_slice(&resp).context("gql json")?;
    if let Some(err) = v.get("errors") {
        bail!("gql errors: {err}");
    }
    let token_obj = v
        .pointer("/data/streamPlaybackAccessToken")
        .ok_or_else(|| anyhow!("no streamPlaybackAccessToken (offline?)"))?;
    if token_obj.is_null() {
        bail!("channel offline or token null");
    }
    let value = token_obj
        .get("value")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("missing token value"))?
        .to_string();
    let signature = token_obj
        .get("signature")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("missing token signature"))?
        .to_string();
    Ok((value, signature))
}

async fn https_request(
    host: &str,
    path_q: &str,
    method: &str,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
) -> Result<Vec<u8>> {
    crate::ensure_rustls_crypto_provider();
    let upstream = TcpStream::connect(format!("{host}:443"))
        .await
        .with_context(|| format!("connect {host}:443"))?;
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let client_cfg = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_cfg));
    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())?;
    let mut tls = connector.connect(server_name, upstream).await?;

    let mut req = format!("{method} {path_q} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(b) = body {
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    req.push_str("\r\n");
    tls.write_all(req.as_bytes()).await?;
    if let Some(b) = body {
        tls.write_all(b).await?;
    }

    let (status_line, hdrs, mut buf) = read_http_headers(&mut tls).await?;
    let status = parse_status(&status_line).unwrap_or(502);
    let mut rest = Vec::new();
    let _ = tls.read_to_end(&mut rest).await;
    buf.extend_from_slice(&rest);
    let body = if is_likely_chunked(&hdrs) {
        decode_chunked(&buf).unwrap_or(buf)
    } else {
        buf
    };
    if !(200..300).contains(&status) {
        let preview = String::from_utf8_lossy(&body[..body.len().min(180)]);
        debug!(%host, status, %preview, "https_request non-2xx");
        bail!("{host} HTTP {status}");
    }
    Ok(body)
}

fn is_likely_chunked(headers: &[(String, String)]) -> bool {
    headers.iter().any(|(k, v)| {
        k.eq_ignore_ascii_case("transfer-encoding") && v.to_ascii_lowercase().contains("chunked")
    })
}

fn decode_chunked(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let end = data[i..].iter().position(|&b| b == b'\n')? + i;
        let line = std::str::from_utf8(&data[i..end]).ok()?.trim();
        let line = line.trim_end_matches('\r');
        let size = usize::from_str_radix(line.split(';').next()?, 16).ok()?;
        i = end + 1;
        if size == 0 {
            break;
        }
        if i + size > data.len() {
            return None;
        }
        out.extend_from_slice(&data[i..i + size]);
        i += size;
        if i + 2 <= data.len() && &data[i..i + 2] == b"\r\n" {
            i += 2;
        } else if i < data.len() && data[i] == b'\n' {
            i += 1;
        }
    }
    Some(out)
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_token_chars() {
        let e = urlencoding_minimal(r#"{"foo":"a b"}"#);
        assert!(e.contains("%7B"));
        assert!(e.contains("%20"));
    }
}
