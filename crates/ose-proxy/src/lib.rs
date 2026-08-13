//! HTTP(S) proxy: Playlist Edge (default) + optional transparent MITM + explicit CONNECT.

mod edge;
mod sni;
mod transparent;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as PollCtx, Poll};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use futures_util::Stream;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use ose_api::{events_json, status_json, StatusHandle};
use ose_cache::{body_hash_hex, CacheKey, PlaylistCache};
use ose_coalesce::{CoalesceError, Singleflight};
use ose_config::{Config, PrefetchPolicyConfig, ProxyMode};
use ose_manifest::{parse, serialize};
use ose_observe::{EngineEvent, EventKind};
use ose_plugin::{
    apply_prefetch_policy, ManifestKind, PluginManager, PrefetchPolicy, RequestMeta,
};
use ose_segment::is_media_segment;
use parking_lot::{Mutex, RwLock};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use transparent::{handle_transparent_tls, spawn_hostlist_refresh, PrefixedStream};

const HEADER_READ_LIMIT: usize = 64 * 1024;
const STREAM_CHUNK: usize = 16 * 1024;

/// rustls 0.23 требует явный CryptoProvider (иначе panic на первом TLS client).
pub fn ensure_rustls_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

type HttpHeaders = Vec<(String, String)>;
type SplitHttpResponse = (Vec<u8>, HttpHeaders, Vec<u8>);

pub type PluginReloader = Arc<dyn Fn() -> anyhow::Result<PluginManager> + Send + Sync>;

pub struct ProxyState {
    pub config: RwLock<Config>,
    pub plugins: RwLock<PluginManager>,
    pub cache: PlaylistCache,
    pub status: StatusHandle,
    pub coalesce: Singleflight,
    pub mitm: Option<Arc<MitmAuthority>>,
    /// Путь к YAML для POST /api/reload.
    pub config_path: Option<std::path::PathBuf>,
    pub reload_plugins: Option<PluginReloader>,
}

impl ProxyState {
    pub fn replace_plugins(&self, plugins: PluginManager) {
        *self.plugins.write() = plugins;
    }

    pub fn replace_config(&self, config: Config) {
        *self.config.write() = config;
    }
}

pub struct MitmAuthority {
    ca_cert_pem: String,
    ca_key: rcgen::KeyPair,
    leaf_cache: Mutex<HashMap<String, Arc<ServerConfig>>>,
}

impl MitmAuthority {
    pub fn generate() -> Result<Self> {
        let mut params = rcgen::CertificateParams::new(vec!["OpenStream Engine CA".into()])?;
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params
            .key_usages
            .push(rcgen::KeyUsagePurpose::KeyCertSign);
        let key = rcgen::KeyPair::generate()?;
        let cert = params.self_signed(&key)?;
        Ok(Self {
            ca_cert_pem: cert.pem(),
            ca_key: key,
            leaf_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn load_or_generate(cert_path: Option<&Path>, key_path: Option<&Path>) -> Result<Self> {
        match (cert_path, key_path) {
            (Some(c), Some(k)) if c.exists() && k.exists() => {
                let cert_pem = fs::read(c)?;
                let key_pem = fs::read(k)?;
                info!(cert = %c.display(), "loaded persistent MITM CA");
                Self::from_pem_files(&cert_pem, &key_pem)
            }
            (Some(c), Some(k)) => {
                let auth = Self::generate()?;
                if let Some(parent) = c.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                fs::write(c, auth.ca_pem())?;
                fs::write(k, auth.ca_key_pem())?;
                info!(cert = %c.display(), key = %k.display(), "wrote new persistent MITM CA");
                Ok(auth)
            }
            _ => {
                info!("using ephemeral MITM CA (set tls.ca_cert/ca_key for persistence)");
                Self::generate()
            }
        }
    }

    fn from_pem_files(cert_pem: &[u8], key_pem: &[u8]) -> Result<Self> {
        let ca_cert_pem = String::from_utf8(cert_pem.to_vec()).map_err(|e| anyhow!("{e}"))?;
        let key_pair = rcgen::KeyPair::from_pem(
            std::str::from_utf8(key_pem).map_err(|e| anyhow!("{e}"))?,
        )
        .map_err(|e| anyhow!("ca key pem: {e}"))?;
        // Validate issuer can be constructed.
        let _issuer = rcgen::Issuer::from_ca_cert_pem(&ca_cert_pem, &key_pair)
            .map_err(|e| anyhow!("ca cert pem: {e}"))?;
        Ok(Self {
            ca_cert_pem,
            ca_key: key_pair,
            leaf_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn ca_pem(&self) -> String {
        self.ca_cert_pem.clone()
    }

    pub fn ca_key_pem(&self) -> String {
        self.ca_key.serialize_pem()
    }

    fn server_config_for_host(&self, host: &str) -> Result<Arc<ServerConfig>> {
        if let Some(cfg) = self.leaf_cache.lock().get(host).cloned() {
            return Ok(cfg);
        }
        let issuer = rcgen::Issuer::from_ca_cert_pem(&self.ca_cert_pem, &self.ca_key)
            .map_err(|e| anyhow!("issuer: {e}"))?;
        let mut params = rcgen::CertificateParams::new(vec![host.to_string()])?;
        params
            .extended_key_usages
            .push(rcgen::ExtendedKeyUsagePurpose::ServerAuth);
        let key = rcgen::KeyPair::generate()?;
        let cert = params
            .signed_by(&key, &issuer)
            .map_err(|e| anyhow!("leaf cert: {e}"))?;
        let cert_der = CertificateDer::from(cert.der().to_vec());
        let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
        let tls_cfg = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)?;
        let arc = Arc::new(tls_cfg);
        self.leaf_cache
            .lock()
            .insert(host.to_string(), arc.clone());
        Ok(arc)
    }
}

pub fn build_state(
    config: Config,
    plugins: PluginManager,
    enable_mitm: bool,
    config_path: Option<std::path::PathBuf>,
    reload_plugins: Option<PluginReloader>,
) -> Result<ProxyState> {
    ensure_rustls_crypto_provider();
    let mitm = if enable_mitm
        && config.mitm
        && matches!(
            config.mode,
            ProxyMode::Transparent | ProxyMode::RedirectWhitelist | ProxyMode::Explicit
        ) {
        let cert = config.tls.ca_cert.as_ref().map(Path::new);
        let key = config.tls.ca_key.as_ref().map(Path::new);
        Some(Arc::new(MitmAuthority::load_or_generate(cert, key)?))
    } else {
        None
    };
    let event_cap = config.observability.event_capacity;
    Ok(ProxyState {
        cache: PlaylistCache::new(Duration::from_secs(config.cache_ttl_secs.max(1))),
        config: RwLock::new(config),
        plugins: RwLock::new(plugins),
        status: StatusHandle::with_event_capacity(event_cap),
        coalesce: Singleflight::new(),
        mitm,
        config_path,
        reload_plugins,
    })
}

pub async fn run(state: Arc<ProxyState>) -> Result<()> {
    apply_mode_side_effects(&state)?;
    spawn_hostlist_refresh(state.clone());

    let mode = state.config.read().mode.clone();
    match mode {
        ProxyMode::Off => info!("mode=off: only /api/* is served"),
        ProxyMode::Edge => info!("mode=edge: Playlist Edge (no client CA); GET /twitch/<channel>"),
        ProxyMode::Transparent | ProxyMode::RedirectWhitelist => {
            info!("mode=transparent: MITM divert (client CA required)")
        }
        ProxyMode::Explicit => info!("mode=explicit: HTTP CONNECT proxy"),
    }

    let addr: SocketAddr = state
        .config
        .read()
        .listen
        .parse()
        .context("invalid listen address")?;
    let listener = TcpListener::bind(addr).await?;
    info!(%addr, ?mode, "streamproxyd listening");
    if let Some(m) = &state.mitm {
        info!(
            ca = %m.ca_pem().lines().next().unwrap_or("PEM"),
            "MITM CA ready (install /etc/openstream/ca.crt on clients for transparent/explicit)"
        );
        debug!("MITM CA PEM:\n{}", m.ca_pem());
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        state.status.bump_streams(1);
        tokio::spawn(async move {
            let status = state.status.clone();
            let result = serve_client(state, stream).await;
            status.bump_streams(-1);
            if let Err(e) = result {
                debug!(error = %e, "client error");
            }
        });
    }
}

/// TLS (0x16) → transparent MITM; иначе HTTP proxy (CONNECT / absolute-URI / API).
async fn serve_client(state: Arc<ProxyState>, mut stream: TcpStream) -> Result<()> {
    let mut first = [0u8; 1];
    let n = stream.read(&mut first).await?;
    if n == 0 {
        return Ok(());
    }
    if first[0] == 0x16 {
        handle_transparent_tls(state, stream, first[0]).await
    } else {
        let prefixed = PrefixedStream::new(vec![first[0]], stream);
        serve_http(state, prefixed).await
    }
}

fn apply_mode_side_effects(state: &ProxyState) -> Result<()> {
    let mode = state.config.read().mode.clone();
    if !mode.uses_transparent_divert() {
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        let (path, hostlist) = {
            let c = state.config.read();
            (c.nft_file.clone(), c.hostlist_file.clone())
        };
        let status = std::process::Command::new("nft")
            .arg("-f")
            .arg(&path)
            .status();
        match status {
            Ok(s) if s.success() => info!(%path, "applied nft transparent divert"),
            Ok(s) => warn!(%path, ?s, "nft apply failed (fail-soft)"),
            Err(e) => warn!(%path, error = %e, "nft not available (fail-soft)"),
        }
        let hl = Path::new(&hostlist);
        if hl.exists() {
            if let Err(e) = transparent::refresh_hls_nft_set(hl) {
                warn!(error = %e, "initial hostlist refresh failed");
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let path = state.config.read().nft_file.clone();
        warn!(%path, "transparent divert nft ignored on non-Linux");
    }
    Ok(())
}

async fn serve_http<S>(state: Arc<ProxyState>, stream: S) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let io = TokioIo::new(stream);
    http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(
            io,
            service_fn(move |req| {
                let state = state.clone();
                async move { dispatch(state, req).await }
            }),
        )
        .with_upgrades()
        .await?;
    Ok(())
}

pub(crate) type RespBody = http_body_util::combinators::UnsyncBoxBody<Bytes, Infallible>;

pub(crate) fn full_body(b: impl Into<Bytes>) -> RespBody {
    Full::new(b.into()).map_err(|e| match e {}).boxed_unsync()
}

async fn dispatch(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<RespBody>, hyper::Error> {
    match dispatch_inner(state, req).await {
        Ok(r) => Ok(r),
        Err(e) => {
            warn!(error = %e, "proxy error");
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(full_body(format!("bad gateway: {e}")))
                .expect("response"))
        }
    }
}

async fn dispatch_inner(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<RespBody>> {
    if req.method() == Method::GET && req.uri().path() == "/api/status" {
        let names = state.plugins.read().plugin_names();
        state
            .status
            .set_plugin_stats(state.plugins.read().aggregate_stats());
        let body = status_json(&state.status, &names);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(full_body(body))
            .expect("response"));
    }

    if req.method() == Method::GET && req.uri().path() == "/api/events" {
        if !state.config.read().observability.events {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body("events disabled"))
                .expect("response"));
        }
        let body = events_json(&state.status);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(full_body(body))
            .expect("response"));
    }

    if req.method() == Method::GET && req.uri().path() == "/metrics" {
        if !state.config.read().observability.metrics {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body("metrics disabled"))
                .expect("response"));
        }
        state
            .status
            .set_plugin_stats(state.plugins.read().aggregate_stats());
        let body = state
            .status
            .metrics_text(state.coalesce.inflight_len() as u64);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
            .body(full_body(body))
            .expect("response"));
    }

    if req.method() == Method::POST && req.uri().path() == "/api/reload" {
        return handle_reload(state).await;
    }

    // Playlist Edge: GET /twitch/<channel> (без CA)
    if req.method() == Method::GET && req.uri().path().starts_with("/twitch/") {
        return edge::handle_twitch_edge(state, req).await;
    }

    if matches!(state.config.read().mode, ProxyMode::Off) {
        return Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(full_body("proxy mode is off"))
            .expect("response"));
    }

    // Edge: nested absolute + API; CONNECT только для explicit/transparent
    if matches!(state.config.read().mode, ProxyMode::Edge) {
        let path_q = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");
        if split_nested_absolute(path_q).is_some() {
            return absolute_http_proxy(state, req).await;
        }
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full_body(
                "mode=edge: use GET /twitch/<channel> or /https://… nested playlist URL",
            ))
            .expect("response"));
    }

    if req.method() == Method::CONNECT {
        return handle_connect(state, req).await;
    }

    absolute_http_proxy(state, req).await
}

async fn handle_reload(state: Arc<ProxyState>) -> Result<Response<RespBody>> {
    let Some(path) = state.config_path.clone() else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full_body("config_path not set"))
            .expect("response"));
    };
    match Config::load(&path) {
        Ok(cfg) => {
            state.replace_config(cfg);
            info!(path = %path.display(), "config reloaded");
            state.status.push_event(EngineEvent::now(
                EventKind::Reload,
                "core",
                "localhost",
                "/api/reload",
                "config reloaded",
                0,
            ));
            if let Some(reload) = &state.reload_plugins {
                match reload() {
                    Ok(pm) => {
                        state.replace_plugins(pm);
                        info!("plugins rebuilt");
                    }
                    Err(e) => warn!(error = %e, "plugin reload failed"),
                }
            }
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(full_body(r#"{"ok":true}"#))
                .expect("response"))
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full_body(format!(r#"{{"ok":false,"error":"{e}"}}"#)))
            .expect("response")),
    }
}

async fn absolute_http_proxy(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<RespBody>> {
    let uri = req.uri().clone();
    let path_q = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();

    // Nested absolute form after master rewrite: /https://cdn/... or path-only https://...
    if let Some((https, host, port, nested_path)) = split_nested_absolute(&path_q) {
        return fetch_and_forward(state, &host, port, &nested_path, https, &req).await;
    }

    let host = uri
        .host()
        .ok_or_else(|| anyhow!("missing host"))?
        .to_string();
    let port = uri.port_u16().unwrap_or(80);
    fetch_and_forward(state, &host, port, &path_q, false, &req).await
}

/// Разбор `/https://host/path` → (https, host, port, path).
pub fn split_nested_absolute(path_q: &str) -> Option<(bool, String, u16, String)> {
    let p = path_q.trim_start_matches('/');
    let (https, rest) = if let Some(r) = p.strip_prefix("https://") {
        (true, r)
    } else {
        let r = p.strip_prefix("http://")?;
        (false, r)
    };
    let (hostport, path) = match rest.split_once('/') {
        Some((h, rest_path)) => (h, format!("/{rest_path}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match hostport.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(if https { 443 } else { 80 })),
        None => (hostport.to_string(), if https { 443 } else { 80 }),
    };
    Some((https, host, port, path))
}

async fn fetch_and_forward(
    state: Arc<ProxyState>,
    host: &str,
    port: u16,
    path_q: &str,
    https: bool,
    req: &Request<Incoming>,
) -> Result<Response<RespBody>> {
    let out_req = build_proxy_request(req, host, path_q);
    if https {
        let upstream = TcpStream::connect(format!("{host}:{port}")).await?;
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let client_cfg = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_cfg));
        let server_name = rustls_pki_types::ServerName::try_from(host.to_string())?;
        let mut upstream_tls = connector.connect(server_name, upstream).await?;
        upstream_tls.write_all(out_req.as_bytes()).await?;
        forward_upstream_response(state, upstream_tls, host, path_q, true).await
    } else {
        let mut upstream = TcpStream::connect(format!("{host}:{port}")).await?;
        upstream.write_all(out_req.as_bytes()).await?;
        forward_upstream_response(state, upstream, host, path_q, false).await
    }
}

fn build_proxy_request(req: &Request<Incoming>, host: &str, path_q: &str) -> String {
    let mut out_req = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        req.method(),
        path_q,
        host
    );
    for (k, v) in req.headers() {
        if k == "proxy-connection" || k == "connection" || k == "host" {
            continue;
        }
        out_req.push_str(&format!("{k}: {}\r\n", v.to_str().unwrap_or("")));
    }
    out_req.push_str("\r\n");
    out_req
}

async fn handle_connect(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<RespBody>> {
    let authority = req
        .uri()
        .authority()
        .map(|a| a.to_string())
        .ok_or_else(|| anyhow!("CONNECT without authority"))?;
    let host = authority
        .split(':')
        .next()
        .unwrap_or(&authority)
        .to_string();

    // MITM только для whitelist: path решим после TLS (m3u8 vs tunnel body).
    let use_mitm = state.mitm.is_some() && host_in_whitelist(&host);
    if use_mitm {
        let state2 = state.clone();
        let authority2 = authority.clone();
        let host2 = host.clone();
        tokio::spawn(async move {
            if let Err(e) = mitm_after_connect(state2, req, authority2, host2).await {
                debug!(error = %e, "mitm failed");
            }
        });
    } else {
        tokio::spawn(async move {
            if let Err(e) = pure_tunnel(req, &authority).await {
                debug!(error = %e, "tunnel failed");
            }
        });
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(full_body(Bytes::new()))
        .expect("response"))
}

async fn pure_tunnel(req: Request<Incoming>, authority: &str) -> Result<()> {
    let upgraded = hyper::upgrade::on(req).await?;
    let mut client = TokioIo::new(upgraded);
    let mut server = TcpStream::connect(authority).await?;
    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
    Ok(())
}

async fn mitm_after_connect(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
    authority: String,
    host: String,
) -> Result<()> {
    let mitm = state.mitm.as_ref().ok_or_else(|| anyhow!("no mitm"))?.clone();
    let upgraded = hyper::upgrade::on(req).await?;
    let client_io = TokioIo::new(upgraded);
    let tls_cfg = mitm.server_config_for_host(&host)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(tls_cfg);
    let client_tls = acceptor.accept(client_io).await?;

    let io = TokioIo::new(client_tls);
    let state2 = state.clone();
    http1::Builder::new()
        .serve_connection(
            io,
            service_fn(move |req: Request<Incoming>| {
                let state = state2.clone();
                let host = host.clone();
                let authority = authority.clone();
                async move {
                    match mitm_forward(state, req, &host, &authority).await {
                        Ok(r) => Ok::<_, hyper::Error>(r),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .body(full_body(e.to_string()))
                            .expect("response")),
                    }
                }
            }),
        )
        .await?;
    Ok(())
}

pub(crate) async fn mitm_forward(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
    host: &str,
    authority: &str,
) -> Result<Response<RespBody>> {
    let path_q = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "/".into());

    let upstream = TcpStream::connect(authority).await?;
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let client_cfg = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_cfg));
    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())?;
    let mut upstream_tls = connector.connect(server_name, upstream).await?;

    let out = build_proxy_request(&req, host, &path_q);
    upstream_tls.write_all(out.as_bytes()).await?;

    forward_upstream_response(state, upstream_tls, host, &path_q, true).await
}

async fn forward_upstream_response<S>(
    state: Arc<ProxyState>,
    mut upstream: S,
    host: &str,
    path_q: &str,
    https: bool,
) -> Result<Response<RespBody>>
where
    S: AsyncRead + Unpin + Send + 'static,
{
    let (status_line, headers, header_end_buf) = read_http_headers(&mut upstream).await?;
    let status = parse_status(&status_line).unwrap_or(502);
    let content_length = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok());

    let scheme = if https { "https" } else { "http" };
    let url = format!("{scheme}://{host}{path_q}");

    let max_manifest = state.config.read().max_manifest_bytes;

    if should_inspect_manifest(path_q) {
        let mut body = header_end_buf;
        read_body_capped(&mut upstream, &mut body, content_length, max_manifest).await?;
        let etag = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("etag"))
            .map(|(_, v)| v.clone());
        let out = process_manifest_if_needed(
            &state,
            host,
            path_q,
            &url,
            &body,
            etag.as_deref(),
            None,
        )
            .await?;
        let content_type = if is_mpd_path(path_q) {
            "application/dash+xml"
        } else {
            "application/vnd.apple.mpegurl"
        };
        return Ok(Response::builder()
            .status(status)
            .header("content-type", content_type)
            .header("content-length", out.len().to_string())
            .body(full_body(out))
            .expect("response"));
    }

    // Сегменты и крупные ответы — streaming без полной буферизации.
    if is_media_segment(path_q) || content_length.unwrap_or(usize::MAX) > max_manifest {
        let body = stream_body_owned(upstream, header_end_buf, content_length);
        let mut builder = Response::builder().status(status);
        for (k, v) in &headers {
            if k.eq_ignore_ascii_case("transfer-encoding")
                || k.eq_ignore_ascii_case("content-length")
            {
                continue;
            }
            builder = builder.header(k.as_str(), v.as_str());
        }
        if let Some(cl) = content_length {
            builder = builder.header("content-length", cl.to_string());
        }
        return Ok(builder.body(body).expect("response"));
    }

    // Небольшие non-m3u8 ответы — capped buffer.
    let max = max_manifest.min(256 * 1024);
    let mut body = header_end_buf;
    read_body_capped(&mut upstream, &mut body, content_length, max).await?;
    let mut builder = Response::builder().status(status);
    for (k, v) in &headers {
        if k.eq_ignore_ascii_case("transfer-encoding") || k.eq_ignore_ascii_case("content-length")
        {
            continue;
        }
        builder = builder.header(k.as_str(), v.as_str());
    }
    Ok(builder
        .header("content-length", body.len().to_string())
        .body(full_body(body))
        .expect("response"))
}

/// Streaming body без аккумуляции всего ответа в RAM.
fn stream_body_owned<S>(mut upstream: S, first: Vec<u8>, content_length: Option<usize>) -> RespBody
where
    S: AsyncRead + Unpin + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(8);
    tokio::spawn(async move {
        let mut sent = 0usize;
        if !first.is_empty() {
            sent += first.len();
            let _ = tx.send(Ok(Bytes::from(first))).await;
        }
        let mut buf = vec![0u8; STREAM_CHUNK];
        loop {
            if let Some(cl) = content_length {
                if sent >= cl {
                    break;
                }
            }
            match upstream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let n = if let Some(cl) = content_length {
                        n.min(cl.saturating_sub(sent))
                    } else {
                        n
                    };
                    if n == 0 {
                        break;
                    }
                    sent += n;
                    if tx
                        .send(Ok(Bytes::copy_from_slice(&buf[..n])))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let stream = ReceiverStream { rx };
    StreamBody::new(stream).boxed_unsync()
}

struct ReceiverStream {
    rx: mpsc::Receiver<Result<Bytes, Infallible>>,
}

impl Stream for ReceiverStream {
    type Item = Result<Frame<Bytes>, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut PollCtx<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.rx).poll_recv(cx) {
            Poll::Ready(Some(Ok(b))) => Poll::Ready(Some(Ok(Frame::data(b)))),
            Poll::Ready(Some(Err(e))) => match e {},
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub(crate) async fn read_http_headers<S: AsyncRead + Unpin>(
    upstream: &mut S,
) -> Result<SplitHttpResponse> {
    let mut buf = Vec::with_capacity(1024);
    let mut tmp = [0u8; 1024];
    loop {
        let n = upstream.read(&mut tmp).await?;
        if n == 0 {
            bail!("upstream closed before headers");
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > HEADER_READ_LIMIT {
            bail!("headers too large");
        }
        if let Some(idx) = find_header_end(&buf) {
            let head = buf[..idx].to_vec();
            let rest = buf[idx + 4..].to_vec();
            let text = std::str::from_utf8(&head).unwrap_or("");
            let mut lines = text.split("\r\n");
            let status = lines.next().unwrap_or("HTTP/1.1 502").as_bytes().to_vec();
            let mut headers = Vec::new();
            for line in lines {
                if let Some((k, v)) = line.split_once(':') {
                    headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            return Ok((status, headers, rest));
        }
    }
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

async fn read_body_capped<S: AsyncRead + Unpin>(
    upstream: &mut S,
    body: &mut Vec<u8>,
    content_length: Option<usize>,
    max: usize,
) -> Result<()> {
    if let Some(cl) = content_length {
        if cl > max {
            bail!("body {cl} exceeds max_manifest_bytes {max}");
        }
        while body.len() < cl {
            let mut tmp = vec![0u8; (cl - body.len()).min(STREAM_CHUNK)];
            let n = upstream.read(&mut tmp).await?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
            if body.len() > max {
                bail!("body exceeds max_manifest_bytes");
            }
        }
        return Ok(());
    }
    let mut tmp = [0u8; STREAM_CHUNK];
    loop {
        let n = upstream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
        if body.len() > max {
            bail!("body exceeds max_manifest_bytes");
        }
    }
    Ok(())
}

pub(crate) async fn process_manifest_if_needed(
    state: &ProxyState,
    host: &str,
    path: &str,
    url: &str,
    body: &[u8],
    etag: Option<&str>,
    // Override proxy_public_url (Edge: Host клиента → rewrite media через nested).
    proxy_base_override: Option<String>,
) -> Result<Vec<u8>> {
    let kind = ManifestKind::from_path(path);
    let proxy_base = proxy_base_override.or_else(|| state.config.read().proxy_public_url.clone());
    // Rewrite меняет тело → ключ кэша должен учитывать proxy_base.
    let cache_url = match proxy_base.as_deref() {
        Some(pb) if !pb.is_empty() => format!("{url}|pb:{pb}"),
        _ => url.to_string(),
    };
    let mut cache_key = CacheKey::from_url(cache_url);
    if let Some(e) = etag {
        cache_key = cache_key.with_etag(e);
    } else {
        cache_key = cache_key.with_body_hash(body_hash_hex(body));
    }
    if let Some((cached, _)) = state.cache.get_key(&cache_key) {
        return Ok(cached.into_bytes());
    }

    let identity = cache_key.identity();
    let body_owned = body.to_vec();
    let host_owned = host.to_string();
    let path_owned = path.to_string();
    let url_owned = url.to_string();

    let out = state
        .coalesce
        .run(identity, || {
            let cache_key = cache_key.clone();
            let body_owned = body_owned;
            let host_owned = host_owned;
            let path_owned = path_owned;
            let url_owned = url_owned;
            let proxy_base = proxy_base.clone();
            async move {
                // Повторная проверка cache после выигрыша singleflight.
                if let Some((cached, _)) = state.cache.get_key(&cache_key) {
                    return Ok(cached.into_bytes());
                }
                process_manifest_inner(
                    state,
                    &host_owned,
                    &path_owned,
                    &url_owned,
                    &body_owned,
                    &cache_key,
                    kind,
                    proxy_base,
                )
                .await
                .map_err(|e| CoalesceError(e.to_string()))
            }
        })
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(out.to_vec())
}

#[allow(clippy::too_many_arguments)]
async fn process_manifest_inner(
    state: &ProxyState,
    host: &str,
    path: &str,
    url: &str,
    body: &[u8],
    cache_key: &CacheKey,
    kind: ManifestKind,
    proxy_base: Option<String>,
) -> Result<Vec<u8>> {
    let prefetch = state.config.read().prefetch_policy.clone();
    let meta = RequestMeta {
        host: host.to_string(),
        path: path.to_string(),
        url: url.to_string(),
        is_manifest: kind.is_manifest(),
        kind,
        proxy_base,
    };
    let plugin = state.plugins.read().find_arc(&meta);

    match kind {
        ManifestKind::Hls => {
            let text = std::str::from_utf8(body).unwrap_or("");
            if !text.contains("#EXTM3U") {
                return Ok(body.to_vec());
            }
            let Some(plugin) = plugin else {
                return Ok(body.to_vec());
            };
            let plugin_name = plugin.name().to_string();
            let manifest = parse(text).map_err(|e| anyhow!(e.to_string()))?;
            let (mut out, outcome) = plugin
                .process_manifest(manifest, &meta)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let policy = match prefetch {
                PrefetchPolicyConfig::Keep => PrefetchPolicy::Keep,
                PrefetchPolicyConfig::StripAll => PrefetchPolicy::StripAll,
                PrefetchPolicyConfig::StripWhenAdsRemoved => PrefetchPolicy::StripWhenAdsRemoved,
            };
            apply_prefetch_policy(&mut out, policy, outcome.ads_found);
            let serialized = serialize(&out);
            state
                .cache
                .put_key(cache_key, serialized.clone(), out.media_sequence());
            state
                .status
                .set_plugin_stats(state.plugins.read().aggregate_stats());
            if outcome.ads_found {
                state.status.push_event(EngineEvent::now(
                    EventKind::AdsStripped,
                    plugin_name,
                    host,
                    path,
                    "hls strip",
                    outcome.segments_removed,
                ));
            }
            Ok(serialized.into_bytes())
        }
        #[cfg(feature = "dash")]
        ManifestKind::Dash => {
            use ose_dash::Mpd;
            let text = std::str::from_utf8(body).unwrap_or("");
            if !text.contains("<MPD") && !text.contains("<mpd") {
                return Ok(body.to_vec());
            }
            let Some(plugin) = plugin else {
                return Ok(body.to_vec());
            };
            let plugin_name = plugin.name().to_string();
            let mpd = Mpd::parse(text).map_err(|e| anyhow!(e.to_string()))?;
            let (out, outcome) = plugin
                .process_mpd(mpd, &meta)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let serialized = out.serialize().map_err(|e| anyhow!(e.to_string()))?;
            state.cache.put_key(cache_key, serialized.clone(), None);
            state
                .status
                .set_plugin_stats(state.plugins.read().aggregate_stats());
            if outcome.ads_found {
                state.status.push_event(EngineEvent::now(
                    EventKind::AdsStripped,
                    plugin_name,
                    host,
                    path,
                    "dash strip",
                    outcome.segments_removed,
                ));
            }
            Ok(serialized.into_bytes())
        }
        #[cfg(not(feature = "dash"))]
        ManifestKind::Dash => {
            let _ = (plugin, prefetch);
            Ok(body.to_vec())
        }
        ManifestKind::Unknown => Ok(body.to_vec()),
    }
}

/// Буферизовать и разбирать как манифест (не как media stream).
pub fn should_inspect_manifest(path: &str) -> bool {
    if is_m3u8_path(path) {
        return true;
    }
    #[cfg(feature = "dash")]
    {
        is_mpd_path(path)
    }
    #[cfg(not(feature = "dash"))]
    {
        false
    }
}

pub(crate) fn parse_status(status_line: &[u8]) -> Option<u16> {
    let s = std::str::from_utf8(status_line).ok()?;
    s.split_whitespace().nth(1)?.parse().ok()
}

pub fn is_m3u8_path(path: &str) -> bool {
    matches!(ManifestKind::from_path(path), ManifestKind::Hls)
}

pub fn is_mpd_path(path: &str) -> bool {
    matches!(ManifestKind::from_path(path), ManifestKind::Dash)
}

pub fn host_in_whitelist(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    // Twitch: только CDN (ttvnw/jtvnw/live-video/weaver). НЕ весь *.twitch.tv —
    // иначе www/gql уходят в MITM и без CA сайт не открывается.
    h.contains("ttvnw.net")
        || h.contains("jtvnw.net")
        || h.contains("live-video.net")
        || h.contains("video-weaver")
        || h.contains("video-edge")
        || h == "vod-secure.twitch.tv"
        || h.ends_with(".vod-secure.twitch.tv")
        || h.contains("kick.com")
        || h.contains("kickusercontent")
        || h.contains("trovo.live")
        || h.contains("trovo.com")
        || h.contains("googlevideo.com")
}

pub fn split_http_response(buf: &[u8]) -> Result<SplitHttpResponse> {
    let idx = find_header_end(buf).ok_or_else(|| anyhow!("invalid HTTP response"))?;
    let head = &buf[..idx];
    let body = buf[idx + 4..].to_vec();
    let text = std::str::from_utf8(head).unwrap_or("");
    let mut lines = text.split("\r\n");
    let status = lines.next().unwrap_or("HTTP/1.1 502").as_bytes().to_vec();
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Ok((status, headers, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m3u8_detection() {
        assert!(is_m3u8_path("/foo/bar.m3u8"));
        assert!(is_m3u8_path("/x.M3U8?token=1"));
        assert!(!is_m3u8_path("/seg.ts"));
        assert!(is_mpd_path("/live.mpd"));
        assert!(!is_mpd_path("/live.m3u8"));
    }

    #[test]
    fn split_response() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhello";
        let (st, hdrs, body) = split_http_response(raw).unwrap();
        assert_eq!(parse_status(&st), Some(200));
        assert_eq!(body, b"hello");
        assert!(hdrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")));
    }

    #[test]
    fn whitelist() {
        assert!(host_in_whitelist("video-weaver.cloudfront.net"));
        assert!(host_in_whitelist("playlist.ttvnw.net"));
        assert!(host_in_whitelist("stream.kick.com"));
        assert!(host_in_whitelist("live.trovo.live"));
        assert!(host_in_whitelist("vod-secure.twitch.tv"));
        assert!(!host_in_whitelist("www.twitch.tv"));
        assert!(!host_in_whitelist("gql.twitch.tv"));
        assert!(!host_in_whitelist("example.com"));
    }

    #[test]
    fn nested_absolute_https() {
        let (https, host, port, path) =
            split_nested_absolute("/https://cdn.example/v1/low.m3u8?t=1").unwrap();
        assert!(https);
        assert_eq!(host, "cdn.example");
        assert_eq!(port, 443);
        assert_eq!(path, "/v1/low.m3u8?t=1");
    }

    #[test]
    fn nested_absolute_http_with_port() {
        let (https, host, port, path) =
            split_nested_absolute("/http://cdn.example:8080/a.m3u8").unwrap();
        assert!(!https);
        assert_eq!(host, "cdn.example");
        assert_eq!(port, 8080);
        assert_eq!(path, "/a.m3u8");
    }
}
