use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};

mod config;
mod limits;
mod proxy;
mod upstream;
mod stats;
mod tunnel;

use config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .json()
        .init();

    let args = Args::parse();
    
    info!("🚀 Riptide Rust Proxy starting...");
    
    // Load configuration
    let config = Config::load(&args.config)?;
    info!("✅ Configuration loaded successfully");
    info!("📋 Config: {} API endpoints, {} upstream providers", 
        config.api.len(), 
        config.upstream.len()
    );

    // Initialize statistics tracker
    let stats = stats::Statistics::new();
    
    // Initialize limits manager
    let limits = limits::LimitsManager::new();
    
    // Initialize upstream manager
    let upstream_mgr = upstream::UpstreamManager::new(config.upstream.clone());
    
    // Start background sync task
    let sync_handle = tokio::spawn({
        let config = config.clone();
        let limits = limits.clone();
        async move {
            background_sync(config, limits).await;
        }
    });

    // Start reporter task
    let reporter_handle = tokio::spawn({
        let config = config.clone();
        let stats = stats.clone();
        let limits = limits.clone();
        async move {
            background_reporter(config, stats, limits).await;
        }
    });

    // Start proxy servers for each API config
    let mut server_handles = vec![];
    
    for api_config in &config.api {
        let ports = parse_port_range(&api_config.ports.userpass)?;
        
        for port in ports {
            let handle = tokio::spawn({
                let config = config.clone();
                let api_config = api_config.clone();
                let stats = stats.clone();
                let limits = limits.clone();
                let upstream_mgr = upstream_mgr.clone();
                
                async move {
                    if let Err(e) = proxy::start_server(
                        port,
                        config,
                        api_config,
                        stats,
                        limits,
                        upstream_mgr,
                    ).await {
                        error!("❌ Server on port {} failed: {}", port, e);
                    }
                }
            });
            
            server_handles.push(handle);
            info!("🌐 Started proxy server on port {}", port);
        }
    }

    info!("✅ All servers started successfully");
    
    // Wait for all tasks
    tokio::select! {
        _ = sync_handle => error!("Sync task terminated"),
        _ = reporter_handle => error!("Reporter task terminated"),
        _ = futures::future::join_all(server_handles) => error!("Server tasks terminated"),
    }

    Ok(())
}

fn parse_port_range(range: &str) -> Result<Vec<u16>> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid port range format: {}", range);
    }
    
    let start: u16 = parts[0].parse()?;
    let end: u16 = parts[1].parse()?;
    
    Ok((start..=end).collect())
}

async fn background_sync(config: Config, limits: limits::LimitsManager) {
    let interval = std::time::Duration::from_secs(config.server.update_interval as u64);
    
    loop {
        tokio::time::sleep(interval).await;
        
        info!("🔄 Starting user sync...");
        
        for api_config in &config.api {
            match fetch_users(&api_config, &config.server.node_name).await {
                Ok(users) => {
                    info!("✅ Fetched {} users from {}", users.len(), api_config.name);
                    
                    // Update limits for each user
                    for user in users {
                        if user.plan.status != "active" {
                            continue;
                        }
                        
                        let key = format!("{}:{}:{}:{}", 
                            user.user_type, 
                            user.username, 
                            user.plan.id,
                            user.user_id
                        );
                        
                        limits.set_limits(
                            &key,
                            user.plan.max_threads.unwrap_or(i32::MAX),
                            user.plan.max_throughput.unwrap_or(i64::MAX),
                            user.plan.max_bytes.unwrap_or(i64::MAX).saturating_sub(user.plan.bytes_used),
                        );
                        
                        // Store credentials
                        limits.add_credential(&user.username, &user.password, key.clone());
                    }
                }
                Err(e) => {
                    error!("❌ Failed to fetch users from {}: {}", api_config.name, e);
                }
            }
        }
        
        info!("✅ User sync completed");
    }
}

async fn background_reporter(
    config: Config,
    stats: stats::Statistics,
    limits: limits::LimitsManager,
) {
    let interval = std::time::Duration::from_secs(config.server.update_interval as u64);
    
    loop {
        tokio::time::sleep(interval).await;
        
        info!("📊 Collecting statistics...");
        
        let reports = stats.collect_reports(&limits);
        
        if !reports.is_empty() {
            info!("📤 Reporting {} key stats", reports.len());
            
            for api_config in &config.api {
                if api_config.legacy {
                    continue;
                }
                
                match report_stats(&api_config, &reports).await {
                    Ok(_) => info!("✅ Reported to {}", api_config.name),
                    Err(e) => error!("❌ Failed to report to {}: {}", api_config.name, e),
                }
            }
        }
        
        stats.reset_counters();
    }
}

#[derive(serde::Deserialize)]
struct User {
    username: String,
    password: String,
    user_id: i64,
    user_type: String,
    plan: Plan,
}

#[derive(serde::Deserialize)]
struct Plan {
    id: i64,
    status: String,
    max_threads: Option<i32>,
    max_throughput: Option<i64>,
    max_bytes: Option<i64>,
    bytes_used: i64,
}

async fn fetch_users(api_config: &config::APIConfig, node_name: &str) -> Result<Vec<User>> {
    let url = format!("{}/riptide?node={}", api_config.base_url, node_name);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .header("api-key", &api_config.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    let users: Vec<User> = response.json().await?;
    Ok(users)
}

async fn report_stats(api_config: &config::APIConfig, reports: &[stats::Report]) -> Result<()> {
    let url = format!("{}/riptide/report", api_config.base_url);
    let client = reqwest::Client::new();
    
    client
        .post(&url)
        .header("api-key", &api_config.api_key)
        .json(reports)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    Ok(())
}

