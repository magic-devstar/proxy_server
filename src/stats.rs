use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct Statistics {
    inner: Arc<StatisticsInner>,
}

struct StatisticsInner {
    bytes_per_key: DashMap<String, i64>,
    last_reset: parking_lot::RwLock<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub key: String,
    pub traffic: i64,
    pub current_threads: i32,
    pub current_throughput: f64, // MB/s
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(StatisticsInner {
                bytes_per_key: DashMap::new(),
                last_reset: parking_lot::RwLock::new(Instant::now()),
            }),
        }
    }
    
    pub fn record_bytes(&self, key: &str, bytes: usize) {
        self.inner.bytes_per_key
            .entry(key.to_string())
            .and_modify(|v| *v += bytes as i64)
            .or_insert(bytes as i64);
    }
    
    pub fn collect_reports(&self, limits: &crate::limits::LimitsManager) -> Vec<Report> {
        let elapsed = self.inner.last_reset.read().elapsed().as_secs_f64();
        let mut reports = Vec::new();
        
        for entry in self.inner.bytes_per_key.iter() {
            let key = entry.key();
            let bytes = *entry.value();
            
            if bytes > 0 {
                let threads = limits.get_active_threads(key);
                let throughput_mbps = if elapsed > 0.0 {
                    (bytes as f64 / elapsed) / 1_000_000.0 // MB/s
                } else {
                    0.0
                };
                
                reports.push(Report {
                    key: key.clone(),
                    traffic: bytes,
                    current_threads: threads,
                    current_throughput: throughput_mbps,
                });
            }
        }
        
        reports
    }
    
    pub fn reset_counters(&self) {
        self.inner.bytes_per_key.clear();
        *self.inner.last_reset.write() = Instant::now();
    }
}

