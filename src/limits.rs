use dashmap::DashMap;
use governor::{Quota, RateLimiter, clock::DefaultClock, state::{InMemoryState, NotKeyed}};
use nonzero_ext::nonzero;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone)]
pub struct LimitsManager {
    inner: Arc<LimitsManagerInner>,
}

struct LimitsManagerInner {
    // Map of username:password -> key
    credentials: DashMap<String, String>,
    
    // Per-key limits
    thread_limits: DashMap<String, i32>,
    throughput_limits: DashMap<String, i64>, // bytes per second
    bandwidth_limits: DashMap<String, i64>,  // total bytes remaining
    
    // Per-key counters
    active_threads: DashMap<String, i32>,
    bytes_used: DashMap<String, i64>,
    
    // Rate limiters per key (for throughput)
    rate_limiters: DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
    
    // Connection rate limiter per key
    conn_rate_limiters: DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

impl LimitsManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(LimitsManagerInner {
                credentials: DashMap::new(),
                thread_limits: DashMap::new(),
                throughput_limits: DashMap::new(),
                bandwidth_limits: DashMap::new(),
                active_threads: DashMap::new(),
                bytes_used: DashMap::new(),
                rate_limiters: DashMap::new(),
                conn_rate_limiters: DashMap::new(),
            }),
        }
    }
    
    pub fn add_credential(&self, username: &str, password: &str, key: String) {
        let cred_key = format!("{}:{}", username, password);
        self.inner.credentials.insert(cred_key, key);
    }
    
    pub fn authenticate(&self, username: &str, password: &str) -> Option<String> {
        let cred_key = format!("{}:{}", username, password);
        self.inner.credentials.get(&cred_key).map(|k| k.clone())
    }
    
    pub fn set_limits(&self, key: &str, threads: i32, throughput_mbps: i64, bandwidth: i64) {
        self.inner.thread_limits.insert(key.to_string(), threads);
        
        // Convert Mbps to bytes per second: Mbps / 8.3 * 0.95 (safety factor)
        let bytes_per_sec = if throughput_mbps == i64::MAX {
            i64::MAX
        } else {
            ((throughput_mbps as f64 / 8.3) * 0.95) as i64 * 1_000_000
        };
        
        self.inner.throughput_limits.insert(key.to_string(), bytes_per_sec);
        self.inner.bandwidth_limits.insert(key.to_string(), bandwidth);
        
        // Create rate limiter for throughput
        if bytes_per_sec != i64::MAX && bytes_per_sec > 0 {
            // Burst size: 8KB
            let burst = NonZeroU32::new(8192).unwrap();
            let quota = Quota::per_second(nonzero!(1u32)).allow_burst(burst);
            let limiter = RateLimiter::direct(quota);
            self.inner.rate_limiters.insert(key.to_string(), Arc::new(limiter));
        }
        
        // Create connection rate limiter (e.g., 100 connections per second per key)
        let conn_quota = Quota::per_second(nonzero!(100u32));
        let conn_limiter = RateLimiter::direct(conn_quota);
        self.inner.conn_rate_limiters.insert(key.to_string(), Arc::new(conn_limiter));
    }
    
    pub fn check_connection_rate(&self, key: &str) -> bool {
        if let Some(limiter) = self.inner.conn_rate_limiters.get(key) {
            limiter.check().is_ok()
        } else {
            true // No limit set
        }
    }
    
    pub fn try_acquire_thread(&self, key: &str) -> bool {
        let limit = self.inner.thread_limits.get(key).map(|v| *v).unwrap_or(i32::MAX);
        
        let mut entry = self.inner.active_threads.entry(key.to_string()).or_insert(0);
        if *entry < limit {
            *entry += 1;
            true
        } else {
            false
        }
    }
    
    pub fn release_thread(&self, key: &str) {
        if let Some(mut entry) = self.inner.active_threads.get_mut(key) {
            *entry = entry.saturating_sub(1);
        }
    }
    
    pub fn get_active_threads(&self, key: &str) -> i32 {
        self.inner.active_threads.get(key).map(|v| *v).unwrap_or(0)
    }
    
    pub async fn wait_for_bandwidth(&self, key: &str, bytes: usize) -> Result<(), String> {
        // Check bandwidth quota
        if let Some(limit) = self.inner.bandwidth_limits.get(key) {
            if *limit != i64::MAX {
                let used = self.inner.bytes_used.entry(key.to_string()).or_insert(0);
                if *used + bytes as i64 > *limit {
                    return Err("Bandwidth quota exceeded".to_string());
                }
            }
        }
        
        // Check throughput rate limit
        if let Some(limiter) = self.inner.rate_limiters.get(key) {
            let throughput_limit = self.inner.throughput_limits
                .get(key)
                .map(|v| *v)
                .unwrap_or(i64::MAX);
            
            if throughput_limit != i64::MAX {
                // Calculate how many "units" this write represents
                // Each unit is roughly bytes_per_sec / rate_limiter_frequency
                let cells = (bytes as u32).max(1);
                
                // Wait for the rate limiter
                let _ = limiter.until_n_ready(NonZeroU32::new(cells.min(8192)).unwrap()).await;
            }
        }
        
        // Update bytes used
        self.inner.bytes_used
            .entry(key.to_string())
            .and_modify(|v| *v += bytes as i64)
            .or_insert(bytes as i64);
        
        Ok(())
    }
    
    // These methods are reserved for future features (control plane sync, debugging)
    #[allow(dead_code)]
    pub fn get_bytes_used(&self, key: &str) -> i64 {
        self.inner.bytes_used.get(key).map(|v| *v).unwrap_or(0)
    }
    
    #[allow(dead_code)]
    pub fn reset_bytes_used(&self, key: &str) {
        self.inner.bytes_used.insert(key.to_string(), 0);
    }
    
    #[allow(dead_code)]
    pub fn get_all_keys(&self) -> Vec<String> {
        self.inner.active_threads.iter().map(|e| e.key().clone()).collect()
    }
}

