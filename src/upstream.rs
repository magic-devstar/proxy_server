use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::debug;

use crate::config::UpstreamProvider;

#[derive(Clone)]
pub struct UpstreamManager {
    providers: Arc<Vec<UpstreamProvider>>,
}

impl UpstreamManager {
    pub fn new(providers: Vec<UpstreamProvider>) -> Self {
        Self {
            providers: Arc::new(providers),
        }
    }
    
    pub fn select(&self, username: &str, password: &str) -> Result<UpstreamConnection> {
        if self.providers.is_empty() {
            anyhow::bail!("No upstream providers configured");
        }
        
        // Parse credentials for mapping parameters
        let params = parse_credential_params(username, password);
        
        // Calculate total weight
        let total_weight: u32 = self.providers.iter().map(|p| p.weight).sum();
        
        if total_weight == 0 {
            anyhow::bail!("Total weight is zero");
        }
        
        // Select provider based on weight
        let mut rng = rand::thread_rng();
        let mut roll: u32 = rng.gen_range(0..total_weight);
        
        let mut selected_provider = None;
        for provider in self.providers.iter() {
            if roll < provider.weight {
                selected_provider = Some(provider);
                break;
            }
            roll -= provider.weight;
        }
        
        let provider = selected_provider.ok_or_else(|| anyhow::anyhow!("No provider selected"))?;
        
        // Select random IP from provider
        if provider.ips.is_empty() {
            anyhow::bail!("Provider has no IPs configured");
        }
        
        let ip_idx = rng.gen_range(0..provider.ips.len());
        let ip_url = &provider.ips[ip_idx];
        
        // Parse IP URL (format: http://host:port or socks5://host:port)
        let (is_socks5, addr) = if ip_url.starts_with("socks5://") {
            (true, ip_url.strip_prefix("socks5://").unwrap())
        } else if ip_url.starts_with("http://") {
            (false, ip_url.strip_prefix("http://").unwrap())
        } else {
            (false, ip_url.as_str())
        };
        
        // Build upstream credentials with parameter mapping
        let upstream_user = build_credential(
            &provider.user,
            &provider.format_in,
            &provider.mapping,
            &provider.separator,
            &params,
        );
        
        let upstream_pass = build_credential(
            &provider.password,
            &provider.format_in,
            &provider.mapping,
            &provider.separator,
            &params,
        );
        
        debug!("🎯 Selected provider: {}, IP: {}, user: {}", 
            provider.name, addr, upstream_user);
        
        Ok(UpstreamConnection {
            ip: addr.to_string(),
            username: upstream_user,
            password: upstream_pass,
            is_socks5,
        })
    }
}

pub struct UpstreamConnection {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub is_socks5: bool,
}

impl UpstreamConnection {
    pub async fn connect(&self, target: &str) -> Result<TcpStream> {
        let stream = TcpStream::connect(&self.ip).await?;
        
        if self.is_socks5 {
            self.connect_socks5(stream, target).await
        } else {
            self.connect_http(stream, target).await
        }
    }
    
    async fn connect_http(&self, mut stream: TcpStream, target: &str) -> Result<TcpStream> {
        // Send CONNECT request
        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.username, self.password)
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
        
        if n == 0 {
            anyhow::bail!("Empty response from upstream");
        }
        
        let response = String::from_utf8_lossy(&buf[..n]);
        
        if !response.contains("200") {
            anyhow::bail!("Upstream connection failed: {}", response);
        }
        
        Ok(stream)
    }
    
    async fn connect_socks5(&self, mut stream: TcpStream, target: &str) -> Result<TcpStream> {
        // SOCKS5 greeting with username/password auth
        stream.write_all(&[0x05, 0x01, 0x02]).await?;
        
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await?;
        
        if buf[0] != 0x05 || buf[1] != 0x02 {
            anyhow::bail!("SOCKS5 auth method not accepted");
        }
        
        // Send username/password
        let mut auth_req = vec![0x01];
        auth_req.push(self.username.len() as u8);
        auth_req.extend_from_slice(self.username.as_bytes());
        auth_req.push(self.password.len() as u8);
        auth_req.extend_from_slice(self.password.as_bytes());
        
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
        
        // Add address type and address
        if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
            req.push(0x01); // IPv4
            req.extend_from_slice(&ip.octets());
        } else {
            req.push(0x03); // Domain
            req.push(host.len() as u8);
            req.extend_from_slice(host.as_bytes());
        }
        
        // Add port
        req.extend_from_slice(&port.to_be_bytes());
        
        stream.write_all(&req).await?;
        
        // Read response
        let mut resp = [0u8; 4];
        stream.read_exact(&mut resp).await?;
        
        if resp[1] != 0x00 {
            anyhow::bail!("SOCKS5 CONNECT failed: {}", resp[1]);
        }
        
        // Read remaining address info
        let atyp = resp[3];
        match atyp {
            0x01 => {
                let mut addr = [0u8; 6]; // 4 bytes IP + 2 bytes port
                stream.read_exact(&mut addr).await?;
            }
            0x03 => {
                let mut len_buf = [0u8; 1];
                stream.read_exact(&mut len_buf).await?;
                let mut domain = vec![0u8; len_buf[0] as usize + 2]; // domain + port
                stream.read_exact(&mut domain).await?;
            }
            0x04 => {
                let mut addr = [0u8; 18]; // 16 bytes IPv6 + 2 bytes port
                stream.read_exact(&mut addr).await?;
            }
            _ => anyhow::bail!("Invalid address type in SOCKS5 response"),
        }
        
        Ok(stream)
    }
}

fn parse_credential_params(username: &str, password: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    // Parse username for dash-separated params (e.g., user-country-us-session-abc)
    let parts: Vec<&str> = username.split('-').collect();
    
    let mut i = 0;
    while i < parts.len() {
        if i + 1 < parts.len() {
            params.insert(parts[i].to_string(), parts[i + 1].to_string());
            i += 2;
        } else {
            i += 1;
        }
    }
    
    params
}

fn build_credential(
    template: &str,
    format_in: &str,
    mapping: &HashMap<String, String>,
    separator: &str,
    params: &HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    
    // If format_in is "username", we build it with separator-joined params
    if format_in == "username" {
        let mut parts = vec![template.to_string()];
        
        // Add mapped parameters
        for (key, mapped) in mapping.iter() {
            if let Some(value) = params.get(key) {
                parts.push(format!("{}{}{}", mapped, separator, value));
            }
        }
        
        result = parts.join(separator);
    }
    
    result
}

fn parse_host_port(target: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = target.rsplitn(2, ':').collect();
    
    if parts.len() == 2 {
        let port = parts[0].parse::<u16>()?;
        let host = parts[1].to_string();
        Ok((host, port))
    } else {
        // Default to port 80
        Ok((target.to_string(), 80))
    }
}

