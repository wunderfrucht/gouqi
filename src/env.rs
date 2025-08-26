//! Environment variable configuration loading
//!
//! This module provides utilities to load configuration from environment variables,
//! supporting various formats and validation.

use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;

use crate::{
    CacheConfig, ConnectionPoolConfig, Credentials, GouqiConfig, MetricsConfig,
    MetricsExportConfig, RateLimitingConfig, RetryConfig, TimeoutConfig,
};

/// Load complete configuration from environment variables
///
/// This function reads various `JIRA_*` environment variables and constructs
/// a complete configuration. If variables are not set, default values are used.
pub fn load_config_from_env() -> GouqiConfig {
    GouqiConfig {
        timeout: load_timeout_config(),
        connection_pool: load_connection_pool_config(),
        cache: load_cache_config(),
        metrics: load_metrics_config(),
        retry: load_retry_config(),
        rate_limiting: load_rate_limiting_config(),
        #[cfg(any(feature = "metrics", feature = "cache"))]
        observability: load_observability_config(),
    }
}

/// Load credentials from environment variables
///
/// Supports the following patterns:
/// - `JIRA_USER` + `JIRA_PASS` for Basic authentication
/// - `JIRA_TOKEN` for Bearer token authentication  
/// - `JIRA_COOKIE` for cookie-based authentication
/// - If none are set, returns Anonymous credentials
pub fn load_credentials_from_env() -> Credentials {
    // Try Bearer token first
    if let Ok(token) = env::var("JIRA_TOKEN") {
        if !token.is_empty() {
            return Credentials::Bearer(token);
        }
    }

    // Try Basic authentication
    if let (Ok(user), Ok(pass)) = (env::var("JIRA_USER"), env::var("JIRA_PASS")) {
        if !user.is_empty() && !pass.is_empty() {
            return Credentials::Basic(user, pass);
        }
    }

    // Try cookie authentication
    if let Ok(cookie) = env::var("JIRA_COOKIE") {
        if !cookie.is_empty() {
            return Credentials::Cookie(cookie);
        }
    }

    // Default to anonymous
    Credentials::Anonymous
}

/// Load host URL from environment variables
///
/// Reads from `JIRA_HOST` or `JIRA_URL` environment variables.
/// Returns None if neither is set.
pub fn load_host_from_env() -> Option<String> {
    env::var("JIRA_HOST")
        .or_else(|_| env::var("JIRA_URL"))
        .ok()
        .and_then(|host| if host.is_empty() { None } else { Some(host) })
}

fn load_timeout_config() -> TimeoutConfig {
    TimeoutConfig {
        default: parse_duration_env("JIRA_TIMEOUT", Duration::from_secs(30)),
        connect: parse_duration_env("JIRA_CONNECT_TIMEOUT", Duration::from_secs(10)),
        read: parse_duration_env("JIRA_READ_TIMEOUT", Duration::from_secs(30)),
    }
}

fn load_connection_pool_config() -> ConnectionPoolConfig {
    ConnectionPoolConfig {
        max_connections_per_host: parse_env("JIRA_MAX_CONNECTIONS", 10),
        idle_timeout: parse_duration_env("JIRA_IDLE_TIMEOUT", Duration::from_secs(30)),
        http2: parse_bool_env("JIRA_HTTP2", true),
        keep_alive_timeout: parse_duration_env("JIRA_KEEP_ALIVE_TIMEOUT", Duration::from_secs(90)),
    }
}

fn load_cache_config() -> CacheConfig {
    let enabled = parse_bool_env("JIRA_CACHE_ENABLED", true);
    let default_ttl = parse_duration_env("JIRA_CACHE_TTL", Duration::from_secs(300));
    let max_entries = parse_env("JIRA_CACHE_MAX_ENTRIES", 1000);

    // Load cache strategies from environment
    let mut strategies = HashMap::new();

    // Known cache strategy endpoints to check for
    let known_endpoints = ["issues", "projects", "search", "users", "components"];

    for endpoint in known_endpoints.iter() {
        let mut strategy_config = None;

        // Check for TTL setting
        let ttl_key = format!("JIRA_CACHE_STRATEGY_{}_TTL", endpoint.to_uppercase());
        if let Ok(ttl_value) = env::var(&ttl_key) {
            if let Some(duration) = parse_duration(&ttl_value) {
                let strategy = strategy_config.get_or_insert(crate::CacheStrategy {
                    ttl: default_ttl,
                    cache_errors: false,
                    use_etag: true,
                });
                strategy.ttl = duration;
            }
        }

        // Check for CACHE_ERRORS setting
        let cache_errors_key = format!(
            "JIRA_CACHE_STRATEGY_{}_CACHE_ERRORS",
            endpoint.to_uppercase()
        );
        if let Ok(cache_errors_value) = env::var(&cache_errors_key) {
            let strategy = strategy_config.get_or_insert(crate::CacheStrategy {
                ttl: default_ttl,
                cache_errors: false,
                use_etag: true,
            });
            strategy.cache_errors = parse_bool(&cache_errors_value).unwrap_or(false);
        }

        // Check for USE_ETAG setting
        let use_etag_key = format!("JIRA_CACHE_STRATEGY_{}_USE_ETAG", endpoint.to_uppercase());
        if let Ok(use_etag_value) = env::var(&use_etag_key) {
            let strategy = strategy_config.get_or_insert(crate::CacheStrategy {
                ttl: default_ttl,
                cache_errors: false,
                use_etag: true,
            });
            strategy.use_etag = parse_bool(&use_etag_value).unwrap_or(true);
        }

        // If we found any configuration for this endpoint, add it to strategies
        if let Some(strategy) = strategy_config {
            strategies.insert(endpoint.to_string(), strategy);
        }
    }

    CacheConfig {
        enabled,
        default_ttl,
        max_entries,
        strategies,
    }
}

fn load_metrics_config() -> MetricsConfig {
    MetricsConfig {
        enabled: parse_bool_env("JIRA_METRICS_ENABLED", true),
        collection_interval: parse_duration_env("JIRA_METRICS_INTERVAL", Duration::from_secs(60)),
        collect_request_times: parse_bool_env("JIRA_METRICS_COLLECT_REQUEST_TIMES", true),
        collect_error_rates: parse_bool_env("JIRA_METRICS_COLLECT_ERROR_RATES", true),
        collect_cache_stats: parse_bool_env("JIRA_METRICS_COLLECT_CACHE_STATS", true),
        export: MetricsExportConfig {
            format: env::var("JIRA_METRICS_EXPORT_FORMAT").unwrap_or_else(|_| "json".to_string()),
            endpoint: env::var("JIRA_METRICS_EXPORT_ENDPOINT").ok(),
            interval: parse_duration_env("JIRA_METRICS_EXPORT_INTERVAL", Duration::from_secs(300)),
        },
    }
}

fn load_retry_config() -> RetryConfig {
    let retry_status_codes = env::var("JIRA_RETRY_STATUS_CODES")
        .unwrap_or_else(|_| "429,500,502,503,504".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse::<u16>().ok())
        .collect();

    RetryConfig {
        max_attempts: parse_env("JIRA_MAX_RETRIES", 3),
        base_delay: parse_duration_env("JIRA_RETRY_BASE_DELAY", Duration::from_millis(100)),
        max_delay: parse_duration_env("JIRA_RETRY_MAX_DELAY", Duration::from_secs(30)),
        backoff_multiplier: parse_env("JIRA_RETRY_BACKOFF", 2.0),
        retry_status_codes,
        retry_on_connection_errors: parse_bool_env("JIRA_RETRY_ON_CONNECTION_ERRORS", true),
    }
}

fn load_rate_limiting_config() -> RateLimitingConfig {
    let mut endpoint_overrides = HashMap::new();

    // Known rate limit endpoints to check for
    let known_endpoints = ["search", "issues", "projects", "users", "components"];

    for endpoint in known_endpoints.iter() {
        let mut override_config = None;

        // Check for RPS setting
        let rps_key = format!("JIRA_RATE_LIMIT_{}_RPS", endpoint.to_uppercase());
        if let Ok(rps_value) = env::var(&rps_key) {
            if let Ok(rps) = rps_value.parse::<f64>() {
                let config = override_config.get_or_insert(crate::RateLimitOverride {
                    requests_per_second: 10.0,
                    burst_capacity: 20,
                });
                config.requests_per_second = rps;
            }
        }

        // Check for BURST setting
        let burst_key = format!("JIRA_RATE_LIMIT_{}_BURST", endpoint.to_uppercase());
        if let Ok(burst_value) = env::var(&burst_key) {
            if let Ok(burst) = burst_value.parse::<u32>() {
                let config = override_config.get_or_insert(crate::RateLimitOverride {
                    requests_per_second: 10.0,
                    burst_capacity: 20,
                });
                config.burst_capacity = burst;
            }
        }

        // If we found any configuration for this endpoint, add it to overrides
        if let Some(config) = override_config {
            endpoint_overrides.insert(endpoint.to_string(), config);
        }
    }

    RateLimitingConfig {
        enabled: parse_bool_env("JIRA_RATE_LIMITING_ENABLED", true),
        requests_per_second: parse_env("JIRA_RATE_LIMIT_RPS", 10.0),
        burst_capacity: parse_env("JIRA_RATE_LIMIT_BURST", 20),
        endpoint_overrides,
    }
}

#[cfg(any(feature = "metrics", feature = "cache"))]
fn load_observability_config() -> crate::observability::ObservabilityConfig {
    crate::observability::ObservabilityConfig {
        enable_tracing: parse_bool_env("JIRA_OBSERVABILITY_TRACING", true),
        enable_metrics: parse_bool_env("JIRA_OBSERVABILITY_METRICS", cfg!(feature = "metrics")),
        enable_caching: parse_bool_env("JIRA_OBSERVABILITY_CACHING", cfg!(feature = "cache")),
        health_check_interval: parse_env("JIRA_OBSERVABILITY_HEALTH_INTERVAL", 30),
        max_error_rate: parse_env("JIRA_OBSERVABILITY_MAX_ERROR_RATE", 10.0),
    }
}

/// Parse environment variable with a default value
fn parse_env<T: FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Parse boolean environment variable with flexible formats
fn parse_bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|s| parse_bool(&s))
        .unwrap_or(default)
}

/// Parse duration from environment variable with support for human-readable formats
fn parse_duration_env(key: &str, default: Duration) -> Duration {
    env::var(key)
        .ok()
        .and_then(|s| parse_duration(&s))
        .unwrap_or(default)
}

/// Parse duration from string with multiple format support
fn parse_duration(s: &str) -> Option<Duration> {
    // Try parsing as seconds first
    if let Ok(secs) = s.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Try parsing as milliseconds if it ends with 'ms'
    if let Some(ms_str) = s.strip_suffix("ms") {
        if let Ok(ms) = ms_str.parse::<u64>() {
            return Some(Duration::from_millis(ms));
        }
    }

    // Try parsing as seconds if it ends with 's'
    if let Some(s_str) = s.strip_suffix('s') {
        if let Ok(secs) = s_str.parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
    }

    // Try parsing as minutes if it ends with 'm'
    if let Some(m_str) = s.strip_suffix('m') {
        if let Ok(mins) = m_str.parse::<u64>() {
            return Some(Duration::from_secs(mins * 60));
        }
    }

    // Try parsing as hours if it ends with 'h'
    if let Some(h_str) = s.strip_suffix('h') {
        if let Ok(hours) = h_str.parse::<u64>() {
            return Some(Duration::from_secs(hours * 3600));
        }
    }

    // Try using humantime crate if available
    #[cfg(feature = "humantime-support")]
    {
        humantime::parse_duration(s).ok()
    }
    #[cfg(not(feature = "humantime-support"))]
    {
        None
    }
}

/// Parse boolean from string with flexible formats
fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => Some(true),
        "false" | "0" | "no" | "off" | "disabled" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_env_var(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    fn remove_test_env_var(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
        assert_eq!(parse_duration("45s"), Some(Duration::from_secs(45)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_duration("invalid"), None);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("enabled"), Some(true));

        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("FALSE"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("disabled"), Some(false));

        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    #[serial_test::serial]
    fn test_load_credentials_anonymous() {
        // Clear any existing environment variables
        remove_test_env_var("JIRA_USER");
        remove_test_env_var("JIRA_PASS");
        remove_test_env_var("JIRA_TOKEN");
        remove_test_env_var("JIRA_COOKIE");

        let creds = load_credentials_from_env();
        matches!(creds, Credentials::Anonymous);
    }

    #[test]
    #[serial_test::serial]
    fn test_load_credentials_basic() {
        set_test_env_var("JIRA_USER", "testuser");
        set_test_env_var("JIRA_PASS", "testpass");
        remove_test_env_var("JIRA_TOKEN");
        remove_test_env_var("JIRA_COOKIE");

        let creds = load_credentials_from_env();
        if let Credentials::Basic(user, pass) = creds {
            assert_eq!(user, "testuser");
            assert_eq!(pass, "testpass");
        } else {
            panic!("Expected Basic credentials");
        }

        // Cleanup
        remove_test_env_var("JIRA_USER");
        remove_test_env_var("JIRA_PASS");
    }

    #[test]
    #[serial_test::serial]
    fn test_load_credentials_bearer() {
        set_test_env_var("JIRA_TOKEN", "bearer-token-123");
        remove_test_env_var("JIRA_USER");
        remove_test_env_var("JIRA_PASS");
        remove_test_env_var("JIRA_COOKIE");

        let creds = load_credentials_from_env();
        if let Credentials::Bearer(token) = creds {
            assert_eq!(token, "bearer-token-123");
        } else {
            panic!("Expected Bearer credentials");
        }

        // Cleanup
        remove_test_env_var("JIRA_TOKEN");
    }
}
