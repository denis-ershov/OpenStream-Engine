//! Прозрачный TLS accept (после nft REDIRECT) + hostlist → nft set.

use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::{anyhow, Context as _, Result};
use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tracing::{debug, warn};

#[cfg(target_os = "linux")]
use tracing::info;

use crate::sni::{extract_sni, tls_record_total_len};
use crate::{host_in_whitelist, ProxyState};

pin_project! {
    /// TcpStream с префиксом уже прочитанных байт (ClientHello / HTTP).
    pub struct PrefixedStream {
        prefix: Vec<u8>,
        prefix_pos: usize,
        #[pin]
        inner: TcpStream,
    }
}

impl PrefixedStream {
    pub fn new(prefix: Vec<u8>, inner: TcpStream) -> Self {
        Self {
            prefix,
            prefix_pos: 0,
            inner,
        }
    }
}

impl AsyncRead for PrefixedStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        if *this.prefix_pos < this.prefix.len() {
            let rest = &this.prefix[*this.prefix_pos..];
            let n = rest.len().min(buf.remaining());
            buf.put_slice(&rest[..n]);
            *this.prefix_pos += n;
            return Poll::Ready(Ok(()));
        }
        this.inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for PrefixedStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }
}

/// SO_ORIGINAL_DST после nft REDIRECT (Linux).
#[cfg(target_os = "linux")]
pub fn original_dst(stream: &TcpStream) -> io::Result<SocketAddr> {
    use std::mem;
    use std::os::fd::AsRawFd;

    let fd = stream.as_raw_fd();
    const SO_ORIGINAL_DST: libc::c_int = 80;
    const IP6T_SO_ORIGINAL_DST: libc::c_int = 80;

    unsafe {
        let mut addr4: libc::sockaddr_in = mem::zeroed();
        let mut len = mem::size_of_val(&addr4) as libc::socklen_t;
        let rc = libc::getsockopt(
            fd,
            libc::SOL_IP,
            SO_ORIGINAL_DST,
            &mut addr4 as *mut _ as *mut libc::c_void,
            &mut len,
        );
        if rc == 0 {
            let ip = Ipv4Addr::from(u32::from_be(addr4.sin_addr.s_addr));
            let port = u16::from_be(addr4.sin_port);
            return Ok(SocketAddr::from((ip, port)));
        }

        let mut addr6: libc::sockaddr_in6 = mem::zeroed();
        len = mem::size_of_val(&addr6) as libc::socklen_t;
        let rc = libc::getsockopt(
            fd,
            libc::SOL_IPV6,
            IP6T_SO_ORIGINAL_DST,
            &mut addr6 as *mut _ as *mut libc::c_void,
            &mut len,
        );
        if rc == 0 {
            let ip = std::net::Ipv6Addr::from(addr6.sin6_addr.s6_addr);
            let port = u16::from_be(addr6.sin6_port);
            return Ok(SocketAddr::from((ip, port)));
        }
    }

    Err(io::Error::new(
        ErrorKind::Unsupported,
        "SO_ORIGINAL_DST unavailable",
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn original_dst(_stream: &TcpStream) -> io::Result<SocketAddr> {
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "SO_ORIGINAL_DST only on Linux",
    ))
}

/// Прочитать ClientHello (с первым уже прочитанным байтом) и вытащить SNI.
pub async fn read_client_hello_prefix(
    stream: &mut TcpStream,
    first: u8,
) -> Result<(Vec<u8>, Option<String>)> {
    let mut buf = vec![0u8; 5];
    buf[0] = first;
    stream.read_exact(&mut buf[1..5]).await?;
    let total = tls_record_total_len(&buf).unwrap_or(512).min(16 * 1024);
    if total > 5 {
        buf.resize(total, 0);
        stream.read_exact(&mut buf[5..]).await?;
    }
    let sni = extract_sni(&buf);
    Ok((buf, sni))
}

/// Transparent TLS: MITM по SNI whitelist или TCP tunnel на original-dst.
pub async fn handle_transparent_tls(
    state: Arc<ProxyState>,
    mut stream: TcpStream,
    first_byte: u8,
) -> Result<()> {
    let dst = match original_dst(&stream) {
        Ok(d) => d,
        Err(e) => {
            debug!(error = %e, "no original dst; treating as local TLS?");
            return Err(anyhow!("transparent requires SO_ORIGINAL_DST: {e}"));
        }
    };

    let (prefix, sni) = read_client_hello_prefix(&mut stream, first_byte).await?;
    let host = sni
        .clone()
        .unwrap_or_else(|| dst.ip().to_string());
    let authority = if let Some(ref s) = sni {
        format!("{s}:{}", dst.port())
    } else {
        dst.to_string()
    };

    let use_mitm = state.mitm.is_some() && host_in_whitelist(&host);
    if use_mitm {
        debug!(%host, %dst, "transparent MITM");
        mitm_transparent(state, stream, prefix, host, authority).await
    } else {
        debug!(%host, %dst, "transparent tunnel (not whitelist)");
        tunnel_with_prefix(stream, prefix, dst).await
    }
}

async fn tunnel_with_prefix(client: TcpStream, prefix: Vec<u8>, dst: SocketAddr) -> Result<()> {
    let mut server = TcpStream::connect(dst).await?;
    server.write_all(&prefix).await?;
    let mut client = client;
    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
    Ok(())
}

async fn mitm_transparent(
    state: Arc<ProxyState>,
    stream: TcpStream,
    prefix: Vec<u8>,
    host: String,
    authority: String,
) -> Result<()> {
    use hyper::body::Incoming;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response, StatusCode};
    use hyper_util::rt::TokioIo;

    use crate::{full_body, mitm_forward};

    let mitm = state.mitm.as_ref().ok_or_else(|| anyhow!("no mitm"))?.clone();
    let tls_cfg = mitm.server_config_for_host(&host)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(tls_cfg);
    let prefixed = PrefixedStream::new(prefix, stream);
    let client_tls = acceptor.accept(prefixed).await?;

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

/// Resolve hostlist file → IPv4 set; apply via `nft`.
pub fn refresh_hls_nft_set(hostlist_path: &Path) -> Result<usize> {
    let text = std::fs::read_to_string(hostlist_path)
        .with_context(|| format!("read hostlist {}", hostlist_path.display()))?;
    let mut ips: Vec<Ipv4Addr> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(ip) = line.parse::<Ipv4Addr>() {
            ips.push(ip);
            continue;
        }
        // domain → A records
        let lookup = format!("{line}:0");
        match lookup.to_socket_addrs() {
            Ok(iter) => {
                for sa in iter {
                    if let SocketAddr::V4(v4) = sa {
                        ips.push(*v4.ip());
                    }
                }
            }
            Err(e) => debug!(domain = %line, error = %e, "hostlist resolve failed"),
        }
    }
    ips.sort_unstable();
    ips.dedup();

    #[cfg(target_os = "linux")]
    {
        // Flush + add. Fail-soft if table missing (nft -f not yet).
        let _ = std::process::Command::new("nft")
            .args(["flush", "set", "inet", "openstream", "openstream_hls"])
            .status();
        if ips.is_empty() {
            warn!("hostlist produced 0 IPv4 addresses");
            return Ok(0);
        }
        let mut args = vec![
            "add".into(),
            "element".into(),
            "inet".into(),
            "openstream".into(),
            "openstream_hls".into(),
            "{".into(),
        ];
        for (i, ip) in ips.iter().enumerate() {
            if i > 0 {
                args.push(",".into());
            }
            args.push(ip.to_string());
        }
        args.push("}".into());
        let status = std::process::Command::new("nft").args(&args).status();
        match status {
            Ok(s) if s.success() => {
                info!(count = ips.len(), "refreshed openstream_hls nft set");
            }
            Ok(s) => warn!(?s, "nft add element failed (fail-soft)"),
            Err(e) => warn!(error = %e, "nft not available (fail-soft)"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        debug!(count = ips.len(), "hostlist resolved (nft skipped on non-Linux)");
    }

    Ok(ips.len())
}

pub fn spawn_hostlist_refresh(state: Arc<ProxyState>) {
    let (path, secs, enabled) = {
        let c = state.config.read();
        (
            c.hostlist_file.clone(),
            c.hostlist_refresh_secs.max(30),
            c.mode.uses_transparent_divert(),
        )
    };
    if !enabled {
        return;
    }
    tokio::spawn(async move {
        loop {
            let p = Path::new(&path);
            if p.exists() {
                if let Err(e) = refresh_hls_nft_set(p) {
                    warn!(error = %e, "hostlist refresh failed");
                }
            } else {
                debug!(%path, "hostlist file missing");
            }
            tokio::time::sleep(Duration::from_secs(secs)).await;
            let still = state.config.read().mode.uses_transparent_divert();
            if !still {
                break;
            }
        }
    });
}
