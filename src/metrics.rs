//! Metrics collection and monitoring
//!
//! This module provides comprehensive metrics collection for monitoring
//! Jira API request performance, cache efficiency, and error rates.

#[cfg(feature = "metrics")]
use std::collections::HashMap;
#[cfg(feature = "metrics")]
use std::sync::Arc;
#[cfg(feature = "metrics")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "metrics")]
use std::time::Duration;

#[cfg(feature = "metrics")]
use once_cell::sync::Lazy;
#[cfg(feature = "metrics")]
use parking_lot::RwLock;
#[cfg(feature = "metrics")]
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Global metrics instance
#[cfg(feature = "metrics")]
pub static METRICS: Lazy<GlobalMetrics> = Lazy::new(|| GlobalMetrics::new());

/// Metrics collection interface
pub trait MetricsCollector: Send + Sync {
    /// Record a request with its duration and success status
    fn record_request(&self, method: &str, endpoint: &str, duration: Duration, success: bool);

    /// Record a cache hit
    fn record_cache_hit(&self, key: &str);

    /// Record a cache miss
    fn record_cache_miss(&self, key: &str);

    /// Record an error occurrence
    fn record_error(&self, error_type: &str);

    /// Get a snapshot of current metrics
    fn get_snapshot(&self) -> MetricsSnapshot;

    /// Reset all metrics (useful for testing)
    fn reset(&self);
}

/// Core metrics implementation
#[cfg(feature = "metrics")]
#[derive(Debug)]
pub struct GlobalMetrics {
    request_count: AtomicU64,
    error_count: AtomicU64,
    total_duration_ms: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    endpoint_metrics: RwLock<HashMap<String, Arc<EndpointMetrics>>>,
}

#[cfg(feature = "metrics")]
impl GlobalMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            endpoint_metrics: RwLock::new(HashMap::new()),
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);

        let avg_duration_ms = if request_count > 0 {
            total_duration / request_count
        } else {
            0
        };

        let success_rate = if request_count > 0 {
            ((request_count - error_count) as f64 / request_count as f64) * 100.0
        } else {
            0.0
        };

        let cache_hit_rate = if (cache_hits + cache_misses) > 0 {
            (cache_hits as f64 / (cache_hits + cache_misses) as f64) * 100.0
        } else {
            0.0
        };

        let endpoint_metrics = self
            .endpoint_metrics
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.snapshot()))
            .collect();

        MetricsSnapshot {
            request_count,
            error_count,
            avg_duration_ms,
            success_rate,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            endpoints: endpoint_metrics,
        }
    }
}

#[cfg(feature = "metrics")]
impl MetricsCollector for GlobalMetrics {
    fn record_request(&self, method: &str, endpoint: &str, duration: Duration, success: bool) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);

        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Record per-endpoint metrics
        let endpoint_key = format!("{} {}", method, endpoint);
        let metrics = {
            let read_guard = self.endpoint_metrics.read();
            read_guard.get(&endpoint_key).cloned()
        };

        let endpoint_metrics = match metrics {
            Some(metrics) => metrics,
            None => {
                let mut write_guard = self.endpoint_metrics.write();
                write_guard
                    .entry(endpoint_key)
                    .or_insert_with(|| Arc::new(EndpointMetrics::new()))
                    .clone()
            }
        };

        endpoint_metrics.record_request(duration, success);

        debug!(
            method = method,
            endpoint = endpoint,
            duration_ms = duration.as_millis(),
            success = success,
            "Request metrics recorded"
        );
    }

    fn record_cache_hit(&self, key: &str) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
        debug!(cache_key = key, "Cache hit recorded");
    }

    fn record_cache_miss(&self, key: &str) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        debug!(cache_key = key, "Cache miss recorded");
    }

    fn record_error(&self, error_type: &str) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        debug!(error_type = error_type, "Error recorded");
    }

    fn get_snapshot(&self) -> MetricsSnapshot {
        self.snapshot()
    }

    fn reset(&self) {
        self.request_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.total_duration_ms.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.endpoint_metrics.write().clear();
        info!("Metrics reset");
    }
}

/// Per-endpoint metrics tracking
#[cfg(feature = "metrics")]
#[derive(Debug)]
pub struct EndpointMetrics {
    request_count: AtomicU64,
    error_count: AtomicU64,
    total_duration_ms: AtomicU64,
    min_duration_ms: AtomicU64,
    max_duration_ms: AtomicU64,
}

#[cfg(feature = "metrics")]
impl EndpointMetrics {
    fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            min_duration_ms: AtomicU64::new(u64::MAX),
            max_duration_ms: AtomicU64::new(0),
        }
    }

    fn record_request(&self, duration: Duration, success: bool) {
        let duration_ms = duration.as_millis() as u64;

        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Update min duration
        self.min_duration_ms
            .fetch_min(duration_ms, Ordering::Relaxed);

        // Update max duration
        self.max_duration_ms
            .fetch_max(duration_ms, Ordering::Relaxed);
    }

    fn snapshot(&self) -> EndpointMetricsSnapshot {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let min_duration = self.min_duration_ms.load(Ordering::Relaxed);
        let max_duration = self.max_duration_ms.load(Ordering::Relaxed);

        let avg_duration_ms = if request_count > 0 {
            total_duration / request_count
        } else {
            0
        };

        let success_rate = if request_count > 0 {
            ((request_count - error_count) as f64 / request_count as f64) * 100.0
        } else {
            0.0
        };

        EndpointMetricsSnapshot {
            request_count,
            error_count,
            avg_duration_ms,
            min_duration_ms: if min_duration == u64::MAX {
                0
            } else {
                min_duration
            },
            max_duration_ms: max_duration,
            success_rate,
        }
    }
}

/// Snapshot of all metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Total number of requests made
    pub request_count: u64,
    /// Total number of errors encountered
    pub error_count: u64,
    /// Average request duration in milliseconds
    pub avg_duration_ms: u64,
    /// Success rate as a percentage
    pub success_rate: f64,
    /// Total cache hits
    pub cache_hits: u64,
    /// Total cache misses
    pub cache_misses: u64,
    /// Cache hit rate as a percentage
    pub cache_hit_rate: f64,
    /// Per-endpoint metrics
    pub endpoints: HashMap<String, EndpointMetricsSnapshot>,
}

/// Snapshot of metrics for a specific endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetricsSnapshot {
    /// Number of requests to this endpoint
    pub request_count: u64,
    /// Number of errors for this endpoint
    pub error_count: u64,
    /// Average request duration in milliseconds
    pub avg_duration_ms: u64,
    /// Minimum request duration in milliseconds
    pub min_duration_ms: u64,
    /// Maximum request duration in milliseconds
    pub max_duration_ms: u64,
    /// Success rate as a percentage
    pub success_rate: f64,
}

impl MetricsSnapshot {
    /// Create an empty metrics snapshot
    pub fn empty() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            avg_duration_ms: 0,
            success_rate: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_rate: 0.0,
            endpoints: HashMap::new(),
        }
    }

    /// Get metrics for a specific endpoint
    pub fn get_endpoint_metrics(&self, endpoint: &str) -> Option<&EndpointMetricsSnapshot> {
        self.endpoints.get(endpoint)
    }

    /// Get the top N endpoints by request count
    pub fn top_endpoints_by_requests(&self, n: usize) -> Vec<(String, &EndpointMetricsSnapshot)> {
        let mut endpoints: Vec<_> = self.endpoints.iter().map(|(k, v)| (k.clone(), v)).collect();

        endpoints.sort_by(|a, b| b.1.request_count.cmp(&a.1.request_count));
        endpoints.truncate(n);
        endpoints
    }

    /// Get endpoints with highest error rates
    pub fn highest_error_endpoints(
        &self,
        min_requests: u64,
    ) -> Vec<(String, &EndpointMetricsSnapshot)> {
        let mut endpoints: Vec<_> = self
            .endpoints
            .iter()
            .filter(|(_, metrics)| metrics.request_count >= min_requests)
            .map(|(k, v)| (k.clone(), v))
            .collect();

        endpoints.sort_by(|a, b| {
            let error_rate_a = (a.1.error_count as f64 / a.1.request_count as f64) * 100.0;
            let error_rate_b = (b.1.error_count as f64 / b.1.request_count as f64) * 100.0;
            error_rate_b
                .partial_cmp(&error_rate_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        endpoints
    }
}

/// Stub implementations for when metrics feature is disabled
#[cfg(not(feature = "metrics"))]
pub struct GlobalMetrics;

#[cfg(not(feature = "metrics"))]
impl GlobalMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::empty()
    }
}

#[cfg(not(feature = "metrics"))]
impl MetricsCollector for GlobalMetrics {
    fn record_request(&self, _method: &str, _endpoint: &str, _duration: Duration, _success: bool) {}
    fn record_cache_hit(&self, _key: &str) {}
    fn record_cache_miss(&self, _key: &str) {}
    fn record_error(&self, _error_type: &str) {}
    fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::empty()
    }
    fn reset(&self) {}
}

#[cfg(not(feature = "metrics"))]
pub static METRICS: GlobalMetrics = GlobalMetrics;
