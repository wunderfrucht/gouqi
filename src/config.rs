//! Configuration management for gouqi Jira client
//!
//! This module provides comprehensive configuration management including:
//! - Loading from files (JSON, YAML, TOML)
//! - Environment variable support
//! - Programmatic configuration
//! - Configuration validation and merging

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::{Error, Result};

/// Main configuration structure for the Jira client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GouqiConfig {
    /// Timeout configuration
    pub timeout: TimeoutConfig,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Metrics collection configuration
    pub metrics: MetricsConfig,
    /// Retry policy configuration
    pub retry: RetryConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Observability configuration
    #[cfg(any(feature = "metrics", feature = "cache"))]
    pub observability: crate::observability::ObservabilityConfig,
}

/// Timeout configuration for various operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default request timeout
    #[serde(with = "humantime_serde")]
    pub default: Duration,
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect: Duration,
    /// Read timeout
    #[serde(with = "humantime_serde")]
    pub read: Duration,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Idle timeout for connections
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    /// Enable HTTP/2
    pub http2: bool,
    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keep_alive_timeout: Duration,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Default TTL for cache entries
    #[serde(with = "humantime_serde")]
    pub default_ttl: Duration,
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Cache strategies per endpoint
    pub strategies: HashMap<String, CacheStrategy>,
}

/// Cache strategy for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategy {
    /// TTL for this endpoint
    #[serde(with = "humantime_serde")]
    pub ttl: Duration,
    /// Whether to cache errors
    pub cache_errors: bool,
    /// Whether to use ETag validation
    pub use_etag: bool,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics collection interval
    #[serde(with = "humantime_serde")]
    pub collection_interval: Duration,
    /// Metrics to collect
    pub collect_request_times: bool,
    pub collect_error_rates: bool,
    pub collect_cache_stats: bool,
    /// Export configuration
    pub export: MetricsExportConfig,
}

/// Metrics export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExportConfig {
    /// Export format (prometheus, json, etc.)
    pub format: String,
    /// Export endpoint
    pub endpoint: Option<String>,
    /// Export interval
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    #[serde(with = "humantime_serde")]
    pub base_delay: Duration,
    /// Maximum delay between retries
    #[serde(with = "humantime_serde")]
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// HTTP status codes that should be retried
    pub retry_status_codes: Vec<u16>,
    /// Whether to retry on connection errors
    pub retry_on_connection_errors: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second limit
    pub requests_per_second: f64,
    /// Burst capacity
    pub burst_capacity: u32,
    /// Rate limit per endpoint overrides
    pub endpoint_overrides: HashMap<String, RateLimitOverride>,
}

/// Rate limit override for specific endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitOverride {
    /// Requests per second for this endpoint
    pub requests_per_second: f64,
    /// Burst capacity for this endpoint
    pub burst_capacity: u32,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default: Duration::from_secs(30),
            connect: Duration::from_secs(10),
            read: Duration::from_secs(30),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            idle_timeout: Duration::from_secs(30),
            http2: true,
            keep_alive_timeout: Duration::from_secs(90),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(300), // 5 minutes
            max_entries: 1000,
            strategies: HashMap::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            collect_request_times: true,
            collect_error_rates: true,
            collect_cache_stats: true,
            export: MetricsExportConfig::default(),
        }
    }
}

impl Default for MetricsExportConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            endpoint: None,
            interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retry_status_codes: vec![429, 500, 502, 503, 504],
            retry_on_connection_errors: true,
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_second: 10.0,
            burst_capacity: 20,
            endpoint_overrides: HashMap::new(),
        }
    }
}

impl Default for GouqiConfig {
    fn default() -> Self {
        Self {
            timeout: TimeoutConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            cache: CacheConfig::default(),
            metrics: MetricsConfig::default(),
            retry: RetryConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            #[cfg(any(feature = "metrics", feature = "cache"))]
            observability: crate::observability::ObservabilityConfig::default(),
        }
    }
}

impl GouqiConfig {
    /// Load configuration from file
    ///
    /// # Panics
    ///
    /// This function will panic if the file cannot be read or contains invalid configuration
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = std::fs::read_to_string(&path).map_err(Error::IO)?;
        let config = match path.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::from_str(&config_str).map_err(Error::Serde)?,
            #[cfg(feature = "yaml")]
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&config_str).map_err(|e| Error::ConfigError {
                    message: format!("YAML parsing error: {}", e),
                })?
            }
            #[cfg(feature = "toml-support")]
            Some("toml") => toml::from_str(&config_str).map_err(|e| Error::ConfigError {
                message: format!("TOML parsing error: {}", e),
            })?,
            Some(ext) => {
                return Err(Error::ConfigError {
                    message: format!("Unsupported config file format: .{}", ext),
                });
            }
            None => {
                return Err(Error::ConfigError {
                    message: "Config file must have an extension (.json, .yaml, .toml)".to_string(),
                });
            }
        };
        Ok(config)
    }

    /// Save configuration to file
    ///
    /// # Panics
    ///
    /// This function will panic if the file cannot be written or the configuration cannot be serialized
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_str = match path.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::to_string_pretty(self).map_err(Error::Serde)?,
            #[cfg(feature = "yaml")]
            Some("yaml") | Some("yml") => {
                serde_yaml::to_string(self).map_err(|e| Error::ConfigError {
                    message: format!("YAML serialization error: {}", e),
                })?
            }
            #[cfg(feature = "toml-support")]
            Some("toml") => toml::to_string_pretty(self).map_err(|e| Error::ConfigError {
                message: format!("TOML serialization error: {}", e),
            })?,
            Some(ext) => {
                return Err(Error::ConfigError {
                    message: format!("Unsupported config file format: .{}", ext),
                });
            }
            None => {
                return Err(Error::ConfigError {
                    message: "Config file must have an extension (.json, .yaml, .toml)".to_string(),
                });
            }
        };

        std::fs::write(path, config_str).map_err(Error::IO)?;
        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge(self, other: Self) -> Self {
        Self {
            timeout: other.timeout,
            connection_pool: other.connection_pool,
            cache: other.cache,
            metrics: other.metrics,
            retry: other.retry,
            rate_limiting: other.rate_limiting,
            #[cfg(any(feature = "metrics", feature = "cache"))]
            observability: other.observability,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate timeout settings
        if self.timeout.default.as_millis() == 0 {
            return Err(Error::ConfigError {
                message: "Default timeout must be greater than 0".to_string(),
            });
        }

        if self.timeout.connect > self.timeout.default {
            return Err(Error::ConfigError {
                message: "Connect timeout cannot be greater than default timeout".to_string(),
            });
        }

        if self.timeout.read > self.timeout.default {
            return Err(Error::ConfigError {
                message: "Read timeout cannot be greater than default timeout".to_string(),
            });
        }

        // Validate connection pool settings
        if self.connection_pool.max_connections_per_host == 0 {
            return Err(Error::ConfigError {
                message: "Connection pool size must be greater than 0".to_string(),
            });
        }

        // Validate retry settings
        if self.retry.max_attempts == 0 {
            return Err(Error::ConfigError {
                message: "Max retry attempts must be greater than 0".to_string(),
            });
        }

        if self.retry.base_delay.as_millis() == 0 {
            return Err(Error::ConfigError {
                message: "Base retry delay must be greater than 0".to_string(),
            });
        }

        if self.retry.max_delay < self.retry.base_delay {
            return Err(Error::ConfigError {
                message: "Max retry delay cannot be less than base delay".to_string(),
            });
        }

        if self.retry.backoff_multiplier <= 0.0 {
            return Err(Error::ConfigError {
                message: "Backoff multiplier must be greater than 0".to_string(),
            });
        }

        // Validate cache settings
        if self.cache.enabled && self.cache.max_entries == 0 {
            return Err(Error::ConfigError {
                message: "Cache max entries must be greater than 0 when caching is enabled"
                    .to_string(),
            });
        }

        if self.cache.enabled && self.cache.default_ttl.as_millis() == 0 {
            return Err(Error::ConfigError {
                message: "Cache default TTL must be greater than 0 when caching is enabled"
                    .to_string(),
            });
        }

        // Validate rate limiting settings
        if self.rate_limiting.enabled {
            if self.rate_limiting.requests_per_second <= 0.0 {
                return Err(Error::ConfigError {
                    message: "Rate limit requests per second must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.burst_capacity == 0 {
                return Err(Error::ConfigError {
                    message: "Rate limit burst capacity must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Create a configuration optimized for high-throughput scenarios
    pub fn high_throughput() -> Self {
        Self {
            timeout: TimeoutConfig {
                default: Duration::from_secs(60),
                connect: Duration::from_secs(5),
                read: Duration::from_secs(60),
            },
            connection_pool: ConnectionPoolConfig {
                max_connections_per_host: 50,
                idle_timeout: Duration::from_secs(60),
                http2: true,
                keep_alive_timeout: Duration::from_secs(120),
            },
            cache: CacheConfig {
                enabled: true,
                default_ttl: Duration::from_secs(120),
                max_entries: 5000,
                strategies: HashMap::new(),
            },
            retry: RetryConfig {
                max_attempts: 5,
                base_delay: Duration::from_millis(50),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 1.5,
                retry_status_codes: vec![429, 500, 502, 503, 504],
                retry_on_connection_errors: true,
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_second: 50.0,
                burst_capacity: 100,
                endpoint_overrides: HashMap::new(),
            },
            ..Default::default()
        }
    }

    /// Create a configuration optimized for low-resource environments
    pub fn low_resource() -> Self {
        Self {
            timeout: TimeoutConfig {
                default: Duration::from_secs(15),
                connect: Duration::from_secs(5),
                read: Duration::from_secs(15),
            },
            connection_pool: ConnectionPoolConfig {
                max_connections_per_host: 2,
                idle_timeout: Duration::from_secs(15),
                http2: false,
                keep_alive_timeout: Duration::from_secs(30),
            },
            cache: CacheConfig {
                enabled: true,
                default_ttl: Duration::from_secs(600),
                max_entries: 100,
                strategies: HashMap::new(),
            },
            retry: RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
                retry_status_codes: vec![429, 500, 502, 503, 504],
                retry_on_connection_errors: true,
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_second: 2.0,
                burst_capacity: 5,
                endpoint_overrides: HashMap::new(),
            },
            metrics: MetricsConfig {
                enabled: false,
                ..Default::default()
            },
            #[cfg(any(feature = "metrics", feature = "cache"))]
            observability: crate::observability::ObservabilityConfig {
                enable_metrics: false,
                enable_caching: false,
                ..Default::default()
            },
        }
    }
}
