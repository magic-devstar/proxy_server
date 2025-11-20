use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

use crate::limits::LimitsManager;
use crate::stats::Statistics;

const BUFFER_SIZE: usize = 8192;

#[cfg(all(target_os = "linux", feature = "io-uring"))]
pub async fn bidirectional_copy(
    client: TcpStream,
    upstream: TcpStream,
    key: String,
    stats: Statistics,
    limits: LimitsManager,
) -> Result<()> {
    // Try to use io_uring, fall back to regular copy on error
    match bidirectional_copy_uring(client, upstream, key.clone(), stats.clone(), limits.clone()).await {
        Ok(_) => Ok(()),
        Err(e) => {
            debug!("⚠️  io_uring failed, falling back to buffered copy: {}", e);
            bidirectional_copy_buffered(client, upstream, key, stats, limits).await
        }
    }
}

#[cfg(all(target_os = "linux", feature = "io-uring"))]
async fn bidirectional_copy_uring(
    client: TcpStream,
    upstream: TcpStream,
    key: String,
    stats: Statistics,
    limits: LimitsManager,
) -> Result<()> {
    use std::os::unix::io::AsRawFd;
    
    let client_fd = client.as_raw_fd();
    let upstream_fd = upstream.as_raw_fd();
    
    // Create pipes for splice operations
    let (pipe_c2u_read, pipe_c2u_write) = nix::unistd::pipe()?;
    let (pipe_u2c_read, pipe_u2c_write) = nix::unistd::pipe()?;
    
    let key1 = key.clone();
    let key2 = key.clone();
    let stats1 = stats.clone();
    let stats2 = stats.clone();
    let limits1 = limits.clone();
    let limits2 = limits.clone();
    
    // Client to upstream
    let h1 = tokio::spawn(async move {
        loop {
            // Splice from client to pipe
            match nix::fcntl::splice(
                client_fd,
                None,
                pipe_c2u_write,
                None,
                BUFFER_SIZE,
                nix::fcntl::SpliceFFlags::empty(),
            ) {
                Ok(0) => break,
                Ok(n) => {
                    // Apply limits
                    if let Err(e) = limits1.wait_for_bandwidth(&key1, n).await {
                        debug!("Bandwidth limit: {}", e);
                        break;
                    }
                    
                    stats1.record_bytes(&key1, n);
                    
                    // Splice from pipe to upstream
                    let mut total = 0;
                    while total < n {
                        match nix::fcntl::splice(
                            pipe_c2u_read,
                            None,
                            upstream_fd,
                            None,
                            n - total,
                            nix::fcntl::SpliceFFlags::empty(),
                        ) {
                            Ok(0) => break,
                            Ok(m) => total += m,
                            Err(_) => break,
                        }
                    }
                    
                    if total != n {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    // Upstream to client
    let h2 = tokio::spawn(async move {
        loop {
            match nix::fcntl::splice(
                upstream_fd,
                None,
                pipe_u2c_write,
                None,
                BUFFER_SIZE,
                nix::fcntl::SpliceFFlags::empty(),
            ) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = limits2.wait_for_bandwidth(&key2, n).await {
                        debug!("Bandwidth limit: {}", e);
                        break;
                    }
                    
                    stats2.record_bytes(&key2, n);
                    
                    let mut total = 0;
                    while total < n {
                        match nix::fcntl::splice(
                            pipe_u2c_read,
                            None,
                            client_fd,
                            None,
                            n - total,
                            nix::fcntl::SpliceFFlags::empty(),
                        ) {
                            Ok(0) => break,
                            Ok(m) => total += m,
                            Err(_) => break,
                        }
                    }
                    
                    if total != n {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    // Wait for both directions
    let _ = tokio::join!(h1, h2);
    
    // Close pipes
    let _ = nix::unistd::close(pipe_c2u_read);
    let _ = nix::unistd::close(pipe_c2u_write);
    let _ = nix::unistd::close(pipe_u2c_read);
    let _ = nix::unistd::close(pipe_u2c_write);
    
    Ok(())
}

#[cfg(not(all(target_os = "linux", feature = "io-uring")))]
pub async fn bidirectional_copy(
    client: TcpStream,
    upstream: TcpStream,
    key: String,
    stats: Statistics,
    limits: LimitsManager,
) -> Result<()> {
    bidirectional_copy_buffered(client, upstream, key, stats, limits).await
}

async fn bidirectional_copy_buffered(
    mut client: TcpStream,
    mut upstream: TcpStream,
    key: String,
    stats: Statistics,
    limits: LimitsManager,
) -> Result<()> {
    let key1 = key.clone();
    let key2 = key.clone();
    let stats1 = stats.clone();
    let stats2 = stats.clone();
    let limits1 = limits.clone();
    let limits2 = limits.clone();
    
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    
    // Client to upstream
    let h1 = tokio::spawn(async move {
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            match client_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    // Apply limits
                    if let Err(e) = limits1.wait_for_bandwidth(&key1, n).await {
                        debug!("Bandwidth limit: {}", e);
                        break;
                    }
                    
                    stats1.record_bytes(&key1, n);
                    
                    if let Err(_) = upstream_write.write_all(&buf[..n]).await {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    // Upstream to client
    let h2 = tokio::spawn(async move {
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            match upstream_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = limits2.wait_for_bandwidth(&key2, n).await {
                        debug!("Bandwidth limit: {}", e);
                        break;
                    }
                    
                    stats2.record_bytes(&key2, n);
                    
                    if let Err(_) = client_write.write_all(&buf[..n]).await {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    // Wait for both directions
    let _ = tokio::join!(h1, h2);
    
    Ok(())
}

