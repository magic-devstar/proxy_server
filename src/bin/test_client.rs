use anyhow::Result;
use clap::Parser;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
#[command(author, version, about = "Riptide Proxy Test Client", long_about = None)]
struct Args {
    /// Proxy address (e.g., 127.0.0.1:8080)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    proxy: String,
    
    /// Username
    #[arg(short, long)]
    username: String,
    
    /// Password
    #[arg(short = 'P', long)]
    password: String,
    
    /// Target to connect to (e.g., google.com:80)
    #[arg(short, long, default_value = "google.com:80")]
    target: String,
    
    /// Number of concurrent connections
    #[arg(short, long, default_value = "1")]
    connections: usize,
    
    /// Duration in seconds
    #[arg(short, long, default_value = "10")]
    duration: u64,
    
    /// Use SOCKS5 instead of HTTP CONNECT
    #[arg(short, long)]
    socks5: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🧪 Riptide Proxy Test Client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📍 Proxy: {}", args.proxy);
    println!("👤 User: {}", args.username);
    println!("🎯 Target: {}", args.target);
    println!("🔗 Connections: {}", args.connections);
    println!("⏱️  Duration: {}s", args.duration);
    println!("📡 Protocol: {}", if args.socks5 { "SOCKS5" } else { "HTTP CONNECT" });
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..args.connections {
        let proxy = args.proxy.clone();
        let username = args.username.clone();
        let password = args.password.clone();
        let target = args.target.clone();
        let duration = args.duration;
        let socks5 = args.socks5;
        
        let handle = tokio::spawn(async move {
            match run_connection(i, &proxy, &username, &password, &target, duration, socks5).await {
                Ok(bytes) => {
                    println!("✅ Connection {} completed: {} bytes transferred", i, bytes);
                    bytes
                }
                Err(e) => {
                    eprintln!("❌ Connection {} failed: {}", i, e);
                    0
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all connections
    let results = futures::future::join_all(handles).await;
    
    let elapsed = start.elapsed();
    let total_bytes: u64 = results.into_iter().filter_map(|r| r.ok()).sum();
    let total_mb = total_bytes as f64 / 1_000_000.0;
    let throughput_mbps = (total_bytes as f64 * 8.0) / (elapsed.as_secs_f64() * 1_000_000.0);
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏱️  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("📦 Total data: {:.2} MB", total_mb);
    println!("🚀 Throughput: {:.2} Mbps", throughput_mbps);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

async fn run_connection(
    id: usize,
    proxy: &str,
    username: &str,
    password: &str,
    target: &str,
    duration: u64,
    socks5: bool,
) -> Result<u64> {
    if socks5 {
        run_socks5_connection(id, proxy, username, password, target, duration).await
    } else {
        run_http_connection(id, proxy, username, password, target, duration).await
    }
}

async fn run_http_connection(
    id: usize,
    proxy: &str,
    username: &str,
    password: &str,
    target: &str,
    duration: u64,
) -> Result<u64> {
    let mut stream = TcpStream::connect(proxy).await?;
    
    // Send CONNECT request
    let auth = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("{}:{}", username, password)
    );
    
    let request = format!(
        "CONNECT {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Proxy-Authorization: Basic {}\r\n\
         \r\n",
        target, target, auth
    );
    
    stream.write_all(request.as_bytes()).await?;
    
    // Read response
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    
    let response = String::from_utf8_lossy(&buf[..n]);
    
    if !response.contains("200") {
        anyhow::bail!("Connection failed: {}", response);
    }
    
    println!("🔗 Connection {} established", id);
    
    // Send and receive data
    let mut total_bytes = 0u64;
    let start = Instant::now();
    let test_data = vec![b'X'; 1024]; // 1KB test payload
    
    while start.elapsed().as_secs() < duration {
        // Send data
        stream.write_all(&test_data).await?;
        total_bytes += test_data.len() as u64;
        
        // Small delay to avoid overwhelming the proxy
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    Ok(total_bytes)
}

async fn run_socks5_connection(
    id: usize,
    proxy: &str,
    username: &str,
    password: &str,
    target: &str,
    duration: u64,
) -> Result<u64> {
    let mut stream = TcpStream::connect(proxy).await?;
    
    // SOCKS5 greeting
    stream.write_all(&[0x05, 0x01, 0x02]).await?;
    
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;
    
    if buf[0] != 0x05 || buf[1] != 0x02 {
        anyhow::bail!("SOCKS5 auth method not accepted");
    }
    
    // Send username/password
    let mut auth_req = vec![0x01];
    auth_req.push(username.len() as u8);
    auth_req.extend_from_slice(username.as_bytes());
    auth_req.push(password.len() as u8);
    auth_req.extend_from_slice(password.as_bytes());
    
    stream.write_all(&auth_req).await?;
    
    let mut auth_resp = [0u8; 2];
    stream.read_exact(&mut auth_resp).await?;
    
    if auth_resp[1] != 0x00 {
        anyhow::bail!("SOCKS5 authentication failed");
    }
    
    // Parse target
    let (host, port) = parse_host_port(target)?;
    
    // Send CONNECT request
    let mut req = vec![0x05, 0x01, 0x00];
    
    if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&ip.octets());
    } else {
        req.push(0x03);
        req.push(host.len() as u8);
        req.extend_from_slice(host.as_bytes());
    }
    
    req.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&req).await?;
    
    // Read response
    let mut resp = [0u8; 10];
    stream.read_exact(&mut resp[..4]).await?;
    
    if resp[1] != 0x00 {
        anyhow::bail!("SOCKS5 CONNECT failed");
    }
    
    // Skip remaining address info based on address type
    match resp[3] {
        0x01 => stream.read_exact(&mut resp[..6]).await?,
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut addr = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut addr).await?;
        }
        0x04 => stream.read_exact(&mut resp[..18]).await?,
        _ => anyhow::bail!("Invalid address type"),
    }
    
    println!("🔗 Connection {} established (SOCKS5)", id);
    
    // Send and receive data
    let mut total_bytes = 0u64;
    let start = Instant::now();
    let test_data = vec![b'X'; 1024];
    
    while start.elapsed().as_secs() < duration {
        stream.write_all(&test_data).await?;
        total_bytes += test_data.len() as u64;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    Ok(total_bytes)
}

fn parse_host_port(target: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = target.rsplitn(2, ':').collect();
    
    if parts.len() == 2 {
        let port = parts[0].parse::<u16>()?;
        let host = parts[1].to_string();
        Ok((host, port))
    } else {
        Ok((target.to_string(), 80))
    }
}

