use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn, error, debug};
use std::sync::Arc;
use base64::Engine;

use crate::config::{Config, APIConfig};
use crate::limits::LimitsManager;
use crate::stats::Statistics;
use crate::upstream::UpstreamManager;
use crate::tunnel;

pub async fn start_server(
    port: u16,
    config: Config,
    api_config: APIConfig,
    stats: Statistics,
    limits: LimitsManager,
    upstream_mgr: UpstreamManager,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🎯 Listening on {}", addr);
    
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let config = config.clone();
                let api_config = api_config.clone();
                let stats = stats.clone();
                let limits = limits.clone();
                let upstream_mgr = upstream_mgr.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(
                        stream,
                        config,
                        api_config,
                        stats,
                        limits,
                        upstream_mgr,
                    ).await {
                        debug!("Connection from {} failed: {}", peer_addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    config: Config,
    api_config: APIConfig,
    stats: Statistics,
    limits: LimitsManager,
    upstream_mgr: UpstreamManager,
) -> Result<()> {
    // Read first bytes to determine protocol
    let mut buf = vec![0u8; 4096];
    let n = stream.peek(&mut buf).await?;
    
    if n == 0 {
        anyhow::bail!("Empty connection");
    }
    
    // Check if it's SOCKS5 (starts with 0x05)
    if buf[0] == 0x05 {
        handle_socks5(stream, config, api_config, stats, limits, upstream_mgr).await
    } else {
        handle_http(stream, config, api_config, stats, limits, upstream_mgr).await
    }
}

async fn handle_http(
    mut stream: TcpStream,
    config: Config,
    _api_config: APIConfig,
    stats: Statistics,
    limits: LimitsManager,
    upstream_mgr: UpstreamManager,
) -> Result<()> {
    // Read HTTP request
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    
    if n == 0 {
        anyhow::bail!("Empty request");
    }
    
    buf.truncate(n);
    
    // Parse HTTP request
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    
    let _status = req.parse(&buf)?;
    
    // Extract Proxy-Authorization header
    let mut username = None;
    let mut password = None;
    
    for header in req.headers.iter() {
        if header.name.eq_ignore_ascii_case("Proxy-Authorization") {
            if let Ok(value) = std::str::from_utf8(header.value) {
                if let Some(basic) = value.strip_prefix("Basic ") {
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(basic) {
                        if let Ok(creds) = String::from_utf8(decoded) {
                            let parts: Vec<&str> = creds.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                username = Some(parts[0].to_string());
                                password = Some(parts[1].to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    let (username, password) = match (username, password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            stream.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"Riptide\"\r\n\r\n").await?;
            anyhow::bail!("No credentials provided");
        }
    };
    
    // Authenticate
    let key = limits.authenticate(&username, &password)
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;
    
    // Check connection rate limit
    if !limits.check_connection_rate(&key) {
        stream.write_all(b"HTTP/1.1 429 Too Many Requests\r\n\r\n").await?;
        anyhow::bail!("Connection rate limit exceeded");
    }
    
    // Try to acquire thread
    if !limits.try_acquire_thread(&key) {
        stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nX-Error: Thread limit exceeded\r\n\r\n").await?;
        anyhow::bail!("Thread limit exceeded");
    }
    
    // Ensure thread is released when done
    let _guard = ThreadGuard {
        limits: limits.clone(),
        key: key.clone(),
    };
    
    let method = req.method.unwrap_or("");
    let path = req.path.unwrap_or("");
    
    if method == "CONNECT" {
        // HTTPS tunneling
        let host = path.to_string();
        
        // Select upstream
        let upstream = upstream_mgr.select(&username, &password)?;
        
        debug!("📡 CONNECT to {} via {}", host, upstream.ip);
        
        // Connect to upstream
        let upstream_conn = upstream.connect(&host).await?;
        
        // Send 200 OK to client
        stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
        
        // Start tunnel
        tunnel::bidirectional_copy(stream, upstream_conn, key, stats, limits).await?;
    } else {
        // Regular HTTP
        let upstream = upstream_mgr.select(&username, &password)?;
        
        debug!("📡 {} {} via {}", method, path, upstream.ip);
        
        // For simplicity in MVP, reject non-CONNECT for now
        stream.write_all(b"HTTP/1.1 501 Not Implemented\r\n\r\nOnly CONNECT method supported in MVP\r\n").await?;
    }
    
    Ok(())
}

async fn handle_socks5(
    mut stream: TcpStream,
    config: Config,
    _api_config: APIConfig,
    stats: Statistics,
    limits: LimitsManager,
    upstream_mgr: UpstreamManager,
) -> Result<()> {
    // SOCKS5 greeting
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;
    
    if buf[0] != 0x05 {
        anyhow::bail!("Invalid SOCKS version");
    }
    
    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    stream.read_exact(&mut methods).await?;
    
    // We require username/password auth (method 0x02)
    if !methods.contains(&0x02) {
        stream.write_all(&[0x05, 0xFF]).await?; // No acceptable methods
        anyhow::bail!("No acceptable auth methods");
    }
    
    // Select username/password auth
    stream.write_all(&[0x05, 0x02]).await?;
    
    // Read auth request
    let mut auth_buf = [0u8; 2];
    stream.read_exact(&mut auth_buf).await?;
    
    if auth_buf[0] != 0x01 {
        anyhow::bail!("Invalid auth version");
    }
    
    let ulen = auth_buf[1] as usize;
    let mut username = vec![0u8; ulen];
    stream.read_exact(&mut username).await?;
    
    let mut plen_buf = [0u8; 1];
    stream.read_exact(&mut plen_buf).await?;
    let plen = plen_buf[0] as usize;
    
    let mut password = vec![0u8; plen];
    stream.read_exact(&mut password).await?;
    
    let username = String::from_utf8(username)?;
    let password = String::from_utf8(password)?;
    
    // Authenticate
    let key = match limits.authenticate(&username, &password) {
        Some(k) => k,
        None => {
            stream.write_all(&[0x01, 0x01]).await?; // Auth failed
            anyhow::bail!("Invalid credentials");
        }
    };
    
    // Auth success
    stream.write_all(&[0x01, 0x00]).await?;
    
    // Check connection rate limit
    if !limits.check_connection_rate(&key) {
        anyhow::bail!("Connection rate limit exceeded");
    }
    
    // Try to acquire thread
    if !limits.try_acquire_thread(&key) {
        anyhow::bail!("Thread limit exceeded");
    }
    
    let _guard = ThreadGuard {
        limits: limits.clone(),
        key: key.clone(),
    };
    
    // Read SOCKS5 request
    let mut req_buf = [0u8; 4];
    stream.read_exact(&mut req_buf).await?;
    
    if req_buf[0] != 0x05 {
        anyhow::bail!("Invalid SOCKS version in request");
    }
    
    let cmd = req_buf[1];
    let atyp = req_buf[3];
    
    if cmd != 0x01 {
        // Only CONNECT supported in MVP
        stream.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        anyhow::bail!("Only CONNECT command supported");
    }
    
    // Parse address
    let host = match atyp {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await?;
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        }
        0x03 => {
            // Domain
            let mut len_buf = [0u8; 1];
            stream.read_exact(&mut len_buf).await?;
            let len = len_buf[0] as usize;
            let mut domain = vec![0u8; len];
            stream.read_exact(&mut domain).await?;
            String::from_utf8(domain)?
        }
        0x04 => {
            // IPv6 - not fully implemented in MVP
            anyhow::bail!("IPv6 not supported in MVP");
        }
        _ => anyhow::bail!("Invalid address type"),
    };
    
    // Read port
    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);
    
    let target = format!("{}:{}", host, port);
    
    // Select upstream
    let upstream = upstream_mgr.select(&username, &password)?;
    
    debug!("📡 SOCKS5 CONNECT to {} via {}", target, upstream.ip);
    
    // Connect to upstream
    let upstream_conn = upstream.connect(&target).await?;
    
    // Send success response
    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
    
    // Start tunnel
    tunnel::bidirectional_copy(stream, upstream_conn, key, stats, limits).await?;
    
    Ok(())
}

struct ThreadGuard {
    limits: LimitsManager,
    key: String,
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        self.limits.release_thread(&self.key);
    }
}

