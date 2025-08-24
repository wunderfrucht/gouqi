use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

use gouqi::{ConnectionPoolConfig, GouqiConfig, TimeoutConfig};

#[test]
fn test_default_config() {
    let config = GouqiConfig::default();

    assert_eq!(config.timeout.default, Duration::from_secs(30));
    assert_eq!(config.timeout.connect, Duration::from_secs(10));
    assert_eq!(config.timeout.read, Duration::from_secs(30));

    assert_eq!(config.connection_pool.max_connections_per_host, 10);
    assert_eq!(config.connection_pool.idle_timeout, Duration::from_secs(30));
    assert!(config.connection_pool.http2);

    assert!(config.cache.enabled);
    assert_eq!(config.cache.default_ttl, Duration::from_secs(300));
    assert_eq!(config.cache.max_entries, 1000);

    assert!(config.metrics.enabled);
    assert_eq!(config.metrics.collection_interval, Duration::from_secs(60));
    assert!(config.metrics.collect_request_times);
    assert!(config.metrics.collect_error_rates);
    assert!(config.metrics.collect_cache_stats);

    assert_eq!(config.retry.max_attempts, 3);
    assert_eq!(config.retry.base_delay, Duration::from_millis(100));
    assert_eq!(config.retry.max_delay, Duration::from_secs(30));
    assert_eq!(config.retry.backoff_multiplier, 2.0);

    assert!(config.rate_limiting.enabled);
    assert_eq!(config.rate_limiting.requests_per_second, 10.0);
    assert_eq!(config.rate_limiting.burst_capacity, 20);
}

#[test]
fn test_config_merge() {
    let base_config = GouqiConfig::default();
    let override_config = GouqiConfig {
        timeout: TimeoutConfig {
            default: Duration::from_secs(60),
            connect: Duration::from_secs(15),
            read: Duration::from_secs(60),
        },
        connection_pool: ConnectionPoolConfig {
            max_connections_per_host: 20,
            ..Default::default()
        },
        ..Default::default()
    };

    let merged = base_config.merge(override_config);

    assert_eq!(merged.timeout.default, Duration::from_secs(60));
    assert_eq!(merged.timeout.connect, Duration::from_secs(15));
    assert_eq!(merged.connection_pool.max_connections_per_host, 20);
}

#[test]
fn test_config_validation_valid() {
    let config = GouqiConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_invalid_timeout() {
    let mut config = GouqiConfig::default();
    config.timeout.default = Duration::from_secs(0);

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Default timeout must be greater than 0")
    );
}

#[test]
fn test_config_validation_invalid_connection_timeout() {
    let mut config = GouqiConfig::default();
    config.timeout.connect = Duration::from_secs(60);
    config.timeout.default = Duration::from_secs(30);

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Connect timeout cannot be greater than default timeout")
    );
}

#[test]
fn test_config_validation_invalid_retry() {
    let mut config = GouqiConfig::default();
    config.retry.max_attempts = 0;

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Max retry attempts must be greater than 0")
    );
}

#[test]
fn test_config_validation_invalid_retry_delay() {
    let mut config = GouqiConfig::default();
    config.retry.max_delay = Duration::from_millis(50);
    config.retry.base_delay = Duration::from_millis(100);

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Max retry delay cannot be less than base delay")
    );
}

#[test]
fn test_high_throughput_config() {
    let config = GouqiConfig::high_throughput();

    assert_eq!(config.timeout.default, Duration::from_secs(60));
    assert_eq!(config.connection_pool.max_connections_per_host, 50);
    assert_eq!(config.cache.max_entries, 5000);
    assert_eq!(config.retry.max_attempts, 5);
    assert_eq!(config.rate_limiting.requests_per_second, 50.0);
    assert_eq!(config.rate_limiting.burst_capacity, 100);
}

#[test]
fn test_low_resource_config() {
    let config = GouqiConfig::low_resource();

    assert_eq!(config.timeout.default, Duration::from_secs(15));
    assert_eq!(config.connection_pool.max_connections_per_host, 2);
    assert_eq!(config.cache.max_entries, 100);
    assert_eq!(config.retry.max_attempts, 2);
    assert_eq!(config.rate_limiting.requests_per_second, 2.0);
    assert_eq!(config.rate_limiting.burst_capacity, 5);
    assert!(!config.metrics.enabled);
}

#[test]
fn test_config_from_json_file() {
    let config_json = r#"
    {
        "timeout": {
            "default": "45s",
            "connect": "15s", 
            "read": "45s"
        },
        "connection_pool": {
            "max_connections_per_host": 20,
            "idle_timeout": "60s",
            "http2": true,
            "keep_alive_timeout": "90s"
        },
        "cache": {
            "enabled": true,
            "default_ttl": "600s",
            "max_entries": 2000,
            "strategies": {}
        },
        "metrics": {
            "enabled": true,
            "collection_interval": "120s",
            "collect_request_times": true,
            "collect_error_rates": true,
            "collect_cache_stats": true,
            "export": {
                "format": "prometheus",
                "endpoint": null,
                "interval": "300s"
            }
        },
        "retry": {
            "max_attempts": 5,
            "base_delay": "200ms",
            "max_delay": "60s",
            "backoff_multiplier": 1.5,
            "retry_status_codes": [429, 500, 502, 503, 504],
            "retry_on_connection_errors": true
        },
        "rate_limiting": {
            "enabled": true,
            "requests_per_second": 25.0,
            "burst_capacity": 50,
            "endpoint_overrides": {}
        }
    }
    "#;

    let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
    temp_file.write_all(config_json.as_bytes()).unwrap();

    let config = GouqiConfig::from_file(temp_file.path()).unwrap();

    assert_eq!(config.timeout.default, Duration::from_secs(45));
    assert_eq!(config.timeout.connect, Duration::from_secs(15));
    assert_eq!(config.connection_pool.max_connections_per_host, 20);
    assert_eq!(config.cache.default_ttl, Duration::from_secs(600));
    assert_eq!(config.cache.max_entries, 2000);
    assert_eq!(config.retry.max_attempts, 5);
    assert_eq!(config.retry.base_delay, Duration::from_millis(200));
    assert_eq!(config.rate_limiting.requests_per_second, 25.0);
    assert_eq!(config.rate_limiting.burst_capacity, 50);
}

#[test]
#[cfg(feature = "yaml")]
fn test_config_from_yaml_file() {
    let config_yaml = r#"
timeout:
  default: "30s"
  connect: "10s"  
  read: "30s"
connection_pool:
  max_connections_per_host: 15
  idle_timeout: "45s"
  http2: false
  keep_alive_timeout: "60s"
cache:
  enabled: false
  default_ttl: "300s"
  max_entries: 500
  strategies: {}
metrics:
  enabled: false
  collection_interval: "60s"
  collect_request_times: false
  collect_error_rates: false
  collect_cache_stats: false
  export:
    format: "json"
    endpoint: null
    interval: "300s"
retry:
  max_attempts: 2
  base_delay: "500ms"
  max_delay: "10s"
  backoff_multiplier: 3.0
  retry_status_codes: [500, 502, 503]
  retry_on_connection_errors: false
rate_limiting:
  enabled: false
  requests_per_second: 5.0
  burst_capacity: 10
  endpoint_overrides: {}
    "#;

    let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
    temp_file.write_all(config_yaml.as_bytes()).unwrap();

    let config = GouqiConfig::from_file(temp_file.path()).unwrap();

    assert_eq!(config.timeout.default, Duration::from_secs(30));
    assert_eq!(config.connection_pool.max_connections_per_host, 15);
    assert!(!config.connection_pool.http2);
    assert!(!config.cache.enabled);
    assert!(!config.metrics.enabled);
    assert_eq!(config.retry.max_attempts, 2);
    assert_eq!(config.retry.backoff_multiplier, 3.0);
    assert!(!config.rate_limiting.enabled);
}

#[test]
#[cfg(feature = "toml-support")]
fn test_config_from_toml_file() {
    let config_toml = r#"
[timeout]
default = "20s"
connect = "5s"
read = "20s"

[connection_pool]
max_connections_per_host = 8
idle_timeout = "30s"
http2 = true
keep_alive_timeout = "45s"

[cache]
enabled = true
default_ttl = "240s"
max_entries = 800

[cache.strategies]

[metrics]
enabled = true
collection_interval = "90s"
collect_request_times = true
collect_error_rates = false
collect_cache_stats = true

[metrics.export]
format = "json"
interval = "600s"

[retry]
max_attempts = 4
base_delay = "150ms"
max_delay = "20s"
backoff_multiplier = 2.5
retry_status_codes = [429, 500, 503, 504]
retry_on_connection_errors = true

[rate_limiting]
enabled = true
requests_per_second = 15.0
burst_capacity = 30

[rate_limiting.endpoint_overrides]
    "#;

    let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
    temp_file.write_all(config_toml.as_bytes()).unwrap();

    let config = GouqiConfig::from_file(temp_file.path()).unwrap();

    assert_eq!(config.timeout.default, Duration::from_secs(20));
    assert_eq!(config.connection_pool.max_connections_per_host, 8);
    assert_eq!(config.cache.default_ttl, Duration::from_secs(240));
    assert_eq!(config.retry.max_attempts, 4);
    assert_eq!(config.retry.backoff_multiplier, 2.5);
    assert_eq!(config.rate_limiting.requests_per_second, 15.0);
}

#[test]
fn test_config_save_to_json_file() {
    let config = GouqiConfig::high_throughput();
    let temp_file = NamedTempFile::with_suffix(".json").unwrap();

    config.save_to_file(temp_file.path()).unwrap();

    // Read it back and verify
    let loaded_config = GouqiConfig::from_file(temp_file.path()).unwrap();
    assert_eq!(loaded_config.timeout.default, config.timeout.default);
    assert_eq!(
        loaded_config.connection_pool.max_connections_per_host,
        config.connection_pool.max_connections_per_host
    );
}

#[test]
fn test_config_invalid_file_format() {
    let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    temp_file.write_all(b"invalid content").unwrap();

    let result = GouqiConfig::from_file(temp_file.path());
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unsupported config file format")
    );
}

#[test]
fn test_config_missing_extension() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"{}").unwrap();

    let result = GouqiConfig::from_file(temp_file.path());
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Config file must have an extension")
    );
}
