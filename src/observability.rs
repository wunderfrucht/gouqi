//! Comprehensive observability infrastructure
//!
//! This module provides a unified observability system that combines metrics collection,
//! caching performance monitoring, request tracing, and health monitoring into a single
//! coherent system for production deployment.

#[cfg(any(feature = "metrics", feature = "cache"))]
use std::sync::Arc;
#[cfg(feature = "metrics")]
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::info;

#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheStats};
#[cfg(feature = "metrics")]
use crate::metrics::{METRICS, MetricsCollector, MetricsSnapshot};

/// Central observability coordinator
pub struct ObservabilitySystem {
    #[cfg(feature = "metrics")]
    metrics: &'static dyn MetricsCollector,
    #[cfg(feature = "cache")]
    cache: Option<Arc<dyn Cache>>,
    health_checker: HealthChecker,
}

impl std::fmt::Debug for ObservabilitySystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObservabilitySystem")
            .field("metrics_enabled", &cfg!(feature = "metrics"))
            .field("cache_enabled", &cfg!(feature = "cache"))
            .field("health_checker", &self.health_checker)
            .finish()
    }
}

impl Default for ObservabilitySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservabilitySystem {
    /// Create a new observability system
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "metrics")]
            metrics: &*METRICS,
            #[cfg(feature = "cache")]
            cache: None,
            health_checker: HealthChecker::new(),
        }
    }

    /// Create observability system with cache monitoring
    #[cfg(feature = "cache")]
    pub fn with_cache(cache: Arc<dyn Cache>) -> Self {
        Self {
            #[cfg(feature = "metrics")]
            metrics: &*METRICS,
            cache: Some(cache),
            health_checker: HealthChecker::new(),
        }
    }

    /// Get comprehensive system health status
    pub fn health_status(&self) -> HealthStatus {
        let mut health = HealthStatus {
            status: "healthy".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metrics: self.get_metrics_health(),
            cache: self.get_cache_health(),
            memory_usage: self.get_memory_usage(),
            uptime: self.health_checker.uptime(),
            request_count: 0,
        };

        #[cfg(feature = "metrics")]
        {
            let snapshot = self.metrics.get_snapshot();
            health.request_count = snapshot.request_count;

            // Mark unhealthy if error rate is too high
            if snapshot.success_rate < 90.0 && snapshot.request_count > 100 {
                health.status = "degraded".to_string();
            }

            if snapshot.success_rate < 50.0 && snapshot.request_count > 50 {
                health.status = "unhealthy".to_string();
            }
        }

        health
    }

    /// Get detailed observability report
    pub fn get_observability_report(&self) -> ObservabilityReport {
        ObservabilityReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            health: self.health_status(),
            #[cfg(feature = "metrics")]
            metrics: Some(self.metrics.get_snapshot()),
            #[cfg(not(feature = "metrics"))]
            metrics: None,
            #[cfg(feature = "cache")]
            cache_stats: self.cache.as_ref().map(|c| c.stats()),
            #[cfg(not(feature = "cache"))]
            cache_stats: None,
            system_info: SystemInfo::collect(),
        }
    }

    /// Record a request for observability tracking
    #[cfg(feature = "metrics")]
    pub fn record_request(&self, method: &str, endpoint: &str, duration: Duration, success: bool) {
        self.metrics
            .record_request(method, endpoint, duration, success);

        if !success {
            self.health_checker.record_error();
        } else {
            self.health_checker.record_success();
        }
    }

    /// Cleanup expired data and optimize performance
    pub fn cleanup(&self) {
        #[cfg(feature = "cache")]
        if let Some(cache) = &self.cache {
            cache.cleanup_expired();
        }

        info!("Observability system cleanup completed");
    }

    /// Reset all metrics and counters
    pub fn reset(&self) {
        #[cfg(feature = "metrics")]
        self.metrics.reset();

        self.health_checker.reset();

        info!("Observability system reset");
    }

    fn get_metrics_health(&self) -> MetricsHealth {
        #[cfg(feature = "metrics")]
        {
            let snapshot = self.metrics.get_snapshot();
            MetricsHealth {
                enabled: true,
                total_requests: snapshot.request_count,
                error_rate: if snapshot.request_count > 0 {
                    (snapshot.error_count as f64 / snapshot.request_count as f64) * 100.0
                } else {
                    0.0
                },
                avg_response_time: snapshot.avg_duration_ms,
            }
        }

        #[cfg(not(feature = "metrics"))]
        MetricsHealth {
            enabled: false,
            total_requests: 0,
            error_rate: 0.0,
            avg_response_time: 0,
        }
    }

    fn get_cache_health(&self) -> CacheHealth {
        #[cfg(feature = "cache")]
        if let Some(cache) = &self.cache {
            let stats = cache.stats();
            return CacheHealth {
                enabled: true,
                total_entries: stats.total_entries,
                active_entries: stats.active_entries,
                hit_rate: if (stats.total_entries) > 0 {
                    (stats.active_entries as f64 / stats.total_entries as f64) * 100.0
                } else {
                    0.0
                },
                memory_usage: stats.total_size_bytes,
            };
        }

        CacheHealth {
            enabled: false,
            total_entries: 0,
            active_entries: 0,
            hit_rate: 0.0,
            memory_usage: 0,
        }
    }

    fn get_memory_usage(&self) -> MemoryUsage {
        // Basic memory usage estimation
        // In a real implementation, you'd use system APIs to get actual memory usage
        MemoryUsage {
            total_mb: 0,     // Would be filled by system info
            used_mb: 0,      // Would be filled by system info
            available_mb: 0, // Would be filled by system info
        }
    }
}

/// Health monitoring component
#[derive(Debug)]
struct HealthChecker {
    start_time: std::time::Instant,
    error_count: std::sync::atomic::AtomicU64,
    success_count: std::sync::atomic::AtomicU64,
}

impl HealthChecker {
    fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            error_count: std::sync::atomic::AtomicU64::new(0),
            success_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    fn record_error(&self) {
        self.error_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_success(&self) {
        self.success_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn reset(&self) {
        self.error_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.success_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Comprehensive health status
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub metrics: MetricsHealth,
    pub cache: CacheHealth,
    pub memory_usage: MemoryUsage,
    pub uptime: u64,
    pub request_count: u64,
}

/// Metrics health information
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsHealth {
    pub enabled: bool,
    pub total_requests: u64,
    pub error_rate: f64,
    pub avg_response_time: u64,
}

/// Cache health information
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHealth {
    pub enabled: bool,
    pub total_entries: usize,
    pub active_entries: usize,
    pub hit_rate: f64,
    pub memory_usage: usize,
}

/// Memory usage information
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
}

/// Complete observability report
#[derive(Debug, Serialize, Deserialize)]
pub struct ObservabilityReport {
    pub timestamp: u64,
    pub health: HealthStatus,
    #[cfg(feature = "metrics")]
    pub metrics: Option<MetricsSnapshot>,
    #[cfg(not(feature = "metrics"))]
    pub metrics: Option<()>,
    #[cfg(feature = "cache")]
    pub cache_stats: Option<CacheStats>,
    #[cfg(not(feature = "cache"))]
    pub cache_stats: Option<()>,
    pub system_info: SystemInfo,
}

/// System information
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub rust_version: String,
    pub library_version: String,
}

impl SystemInfo {
    fn collect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            library_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable comprehensive logging
    pub enable_tracing: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable response caching
    pub enable_caching: bool,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Maximum error rate before marking unhealthy
    pub max_error_rate: f64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            enable_metrics: cfg!(feature = "metrics"),
            enable_caching: cfg!(feature = "cache"),
            health_check_interval: 30,
            max_error_rate: 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_system_creation() {
        let obs = ObservabilitySystem::new();
        let health = obs.health_status();
        assert_eq!(health.status, "healthy");
        assert!(health.uptime >= 0);
    }

    #[test]
    fn test_health_status_serialization() {
        let obs = ObservabilitySystem::new();
        let health = obs.health_status();

        let json = serde_json::to_string(&health).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(health.status, deserialized.status);
    }

    #[test]
    fn test_observability_report() {
        let obs = ObservabilitySystem::new();
        let report = obs.get_observability_report();

        assert!(report.timestamp > 0);
        assert_eq!(report.health.status, "healthy");
        assert_eq!(
            report.system_info.library_version,
            env!("CARGO_PKG_VERSION")
        );
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo::collect();
        assert!(!info.os.is_empty());
        assert!(!info.architecture.is_empty());
        assert!(!info.library_version.is_empty());
    }

    #[test]
    fn test_observability_config_defaults() {
        let config = ObservabilityConfig::default();
        assert!(config.enable_tracing);
        assert_eq!(config.health_check_interval, 30);
        assert_eq!(config.max_error_rate, 10.0);
    }
}
