use std::env;
use std::time::Duration;

use gouqi::Credentials;
use gouqi::env::{load_config_from_env, load_credentials_from_env, load_host_from_env};
use serial_test::serial;

// Helper functions for safe environment variable operations in tests
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

fn clear_jira_env_vars() {
    let jira_vars = [
        "JIRA_HOST",
        "JIRA_URL",
        "JIRA_USER",
        "JIRA_PASS",
        "JIRA_TOKEN",
        "JIRA_COOKIE",
        "JIRA_TIMEOUT",
        "JIRA_CONNECT_TIMEOUT",
        "JIRA_READ_TIMEOUT",
        "JIRA_MAX_CONNECTIONS",
        "JIRA_IDLE_TIMEOUT",
        "JIRA_HTTP2",
        "JIRA_KEEP_ALIVE_TIMEOUT",
        "JIRA_CACHE_ENABLED",
        "JIRA_CACHE_TTL",
        "JIRA_CACHE_MAX_ENTRIES",
        "JIRA_METRICS_ENABLED",
        "JIRA_METRICS_INTERVAL",
        "JIRA_METRICS_COLLECT_REQUEST_TIMES",
        "JIRA_METRICS_COLLECT_ERROR_RATES",
        "JIRA_METRICS_COLLECT_CACHE_STATS",
        "JIRA_METRICS_EXPORT_FORMAT",
        "JIRA_METRICS_EXPORT_ENDPOINT",
        "JIRA_METRICS_EXPORT_INTERVAL",
        "JIRA_MAX_RETRIES",
        "JIRA_RETRY_BASE_DELAY",
        "JIRA_RETRY_MAX_DELAY",
        "JIRA_RETRY_BACKOFF",
        "JIRA_RETRY_STATUS_CODES",
        "JIRA_RETRY_ON_CONNECTION_ERRORS",
        "JIRA_RATE_LIMITING_ENABLED",
        "JIRA_RATE_LIMIT_RPS",
        "JIRA_RATE_LIMIT_BURST",
    ];

    for var in jira_vars.iter() {
        remove_test_env_var(var);
    }
}

#[test]
#[serial]
fn test_load_credentials_anonymous() {
    clear_jira_env_vars();

    let creds = load_credentials_from_env();
    assert!(matches!(creds, Credentials::Anonymous));
}

#[test]
#[serial]
fn test_load_credentials_basic() {
    clear_jira_env_vars();

    set_test_env_var("JIRA_USER", "testuser");
    set_test_env_var("JIRA_PASS", "testpass");

    let creds = load_credentials_from_env();
    if let Credentials::Basic(user, pass) = creds {
        assert_eq!(user, "testuser");
        assert_eq!(pass, "testpass");
    } else {
        panic!("Expected Basic credentials, got {:?}", creds);
    }

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_credentials_bearer() {
    clear_jira_env_vars();

    set_test_env_var("JIRA_TOKEN", "bearer-token-123");

    let creds = load_credentials_from_env();
    if let Credentials::Bearer(token) = creds {
        assert_eq!(token, "bearer-token-123");
    } else {
        panic!("Expected Bearer credentials, got {:?}", creds);
    }

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_credentials_cookie() {
    clear_jira_env_vars();

    set_test_env_var("JIRA_COOKIE", "JSESSIONID=ABC123");

    let creds = load_credentials_from_env();
    if let Credentials::Cookie(cookie) = creds {
        assert_eq!(cookie, "JSESSIONID=ABC123");
    } else {
        panic!("Expected Cookie credentials, got {:?}", creds);
    }

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_credentials_priority_order() {
    clear_jira_env_vars();

    // Set all types - Bearer should have priority
    set_test_env_var("JIRA_TOKEN", "bearer-token");
    set_test_env_var("JIRA_USER", "user");
    set_test_env_var("JIRA_PASS", "pass");
    set_test_env_var("JIRA_COOKIE", "cookie");

    let creds = load_credentials_from_env();
    assert!(matches!(creds, Credentials::Bearer(_)));

    // Remove bearer, should fall back to Basic
    remove_test_env_var("JIRA_TOKEN");
    let creds = load_credentials_from_env();
    assert!(matches!(creds, Credentials::Basic(_, _)));

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_credentials_empty_values() {
    clear_jira_env_vars();

    // Test empty token
    set_test_env_var("JIRA_TOKEN", "");
    set_test_env_var("JIRA_USER", "user");
    set_test_env_var("JIRA_PASS", "");

    let creds = load_credentials_from_env();
    // Should fall back to anonymous since token and pass are empty
    assert!(matches!(creds, Credentials::Anonymous));

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_host_from_env() {
    clear_jira_env_vars();

    // Test JIRA_HOST
    set_test_env_var("JIRA_HOST", "https://host.example.com");
    let host = load_host_from_env();
    assert_eq!(host, Some("https://host.example.com".to_string()));

    // Test JIRA_URL fallback
    remove_test_env_var("JIRA_HOST");
    set_test_env_var("JIRA_URL", "https://url.example.com");
    let host = load_host_from_env();
    assert_eq!(host, Some("https://url.example.com".to_string()));

    // Test empty value
    set_test_env_var("JIRA_HOST", "");
    let host = load_host_from_env();
    assert_eq!(host, None);

    // Test no variables set
    clear_jira_env_vars();
    let host = load_host_from_env();
    assert_eq!(host, None);

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_config_basic() {
    clear_jira_env_vars();

    // Set basic config values
    set_test_env_var("JIRA_TIMEOUT", "45");
    set_test_env_var("JIRA_MAX_RETRIES", "5");
    set_test_env_var("JIRA_MAX_CONNECTIONS", "20");

    let config = load_config_from_env();

    assert_eq!(config.timeout.default, Duration::from_secs(45));
    assert_eq!(config.retry.max_attempts, 5);
    assert_eq!(config.connection_pool.max_connections_per_host, 20);

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_config_duration_formats() {
    clear_jira_env_vars();

    // Test different duration formats
    set_test_env_var("JIRA_TIMEOUT", "30s");
    set_test_env_var("JIRA_CONNECT_TIMEOUT", "5000ms");
    set_test_env_var("JIRA_READ_TIMEOUT", "2m");

    let config = load_config_from_env();

    assert_eq!(config.timeout.default, Duration::from_secs(30));
    assert_eq!(config.timeout.connect, Duration::from_millis(5000));
    assert_eq!(config.timeout.read, Duration::from_secs(120)); // 2 minutes

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_config_boolean_formats() {
    clear_jira_env_vars();

    // Test various boolean formats
    set_test_env_var("JIRA_HTTP2", "true");
    set_test_env_var("JIRA_CACHE_ENABLED", "1");
    set_test_env_var("JIRA_METRICS_ENABLED", "yes");
    set_test_env_var("JIRA_RATE_LIMITING_ENABLED", "on");

    let config = load_config_from_env();

    assert!(config.connection_pool.http2);
    assert!(config.cache.enabled);
    assert!(config.metrics.enabled);
    assert!(config.rate_limiting.enabled);

    // Test false values
    set_test_env_var("JIRA_HTTP2", "false");
    set_test_env_var("JIRA_CACHE_ENABLED", "0");
    set_test_env_var("JIRA_METRICS_ENABLED", "no");
    set_test_env_var("JIRA_RATE_LIMITING_ENABLED", "disabled");

    let config = load_config_from_env();

    assert!(!config.connection_pool.http2);
    assert!(!config.cache.enabled);
    assert!(!config.metrics.enabled);
    assert!(!config.rate_limiting.enabled);

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_load_config_invalid_values() {
    clear_jira_env_vars();

    // Test invalid values fall back to defaults
    set_test_env_var("JIRA_RETRY_BACKOFF", "2.5");
    let config = load_config_from_env();
    assert_eq!(config.retry.backoff_multiplier, 2.5);

    // Invalid number should fall back to default
    set_test_env_var("JIRA_MAX_RETRIES", "not_a_number");
    let config = load_config_from_env();
    assert_eq!(config.retry.max_attempts, 3); // default

    // Invalid float should fall back to default
    set_test_env_var("JIRA_RATE_LIMIT_RPS", "not_a_float");
    let config = load_config_from_env();
    assert_eq!(config.rate_limiting.requests_per_second, 10.0); // default

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_retry_status_codes_parsing() {
    clear_jira_env_vars();

    // Test valid status codes
    set_test_env_var("JIRA_RETRY_STATUS_CODES", "429,500,502,503,504");
    let config = load_config_from_env();
    assert_eq!(
        config.retry.retry_status_codes,
        vec![429, 500, 502, 503, 504]
    );

    // Test with invalid codes mixed in
    set_test_env_var("JIRA_RETRY_STATUS_CODES", "429,invalid,500,999999");
    let config = load_config_from_env();
    assert_eq!(config.retry.retry_status_codes, vec![429, 500]); // Only valid ones

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_cache_strategies_parsing() {
    clear_jira_env_vars();

    // Set up cache strategy environment variables
    set_test_env_var("JIRA_CACHE_STRATEGY_ISSUES_TTL", "300s");
    set_test_env_var("JIRA_CACHE_STRATEGY_ISSUES_CACHE_ERRORS", "true");
    set_test_env_var("JIRA_CACHE_STRATEGY_ISSUES_USE_ETAG", "false");
    set_test_env_var("JIRA_CACHE_STRATEGY_PROJECTS_TTL", "600s");
    set_test_env_var("JIRA_CACHE_STRATEGY_PROJECTS_CACHE_ERRORS", "false");

    let config = load_config_from_env();

    assert!(config.cache.strategies.contains_key("issues"));
    assert!(config.cache.strategies.contains_key("projects"));

    let issues_strategy = &config.cache.strategies["issues"];
    assert_eq!(issues_strategy.ttl, Duration::from_secs(300));
    assert!(issues_strategy.cache_errors);
    assert!(!issues_strategy.use_etag);

    let projects_strategy = &config.cache.strategies["projects"];
    assert_eq!(projects_strategy.ttl, Duration::from_secs(600));
    assert!(!projects_strategy.cache_errors);
    assert!(projects_strategy.use_etag); // default

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_rate_limit_overrides_parsing() {
    clear_jira_env_vars();

    // Set up rate limit override environment variables
    set_test_env_var("JIRA_RATE_LIMIT_SEARCH_RPS", "5.0");
    set_test_env_var("JIRA_RATE_LIMIT_SEARCH_BURST", "15");
    set_test_env_var("JIRA_RATE_LIMIT_ISSUES_RPS", "20.0");
    set_test_env_var("JIRA_RATE_LIMIT_ISSUES_BURST", "50");

    let config = load_config_from_env();

    assert!(
        config
            .rate_limiting
            .endpoint_overrides
            .contains_key("search")
    );
    assert!(
        config
            .rate_limiting
            .endpoint_overrides
            .contains_key("issues")
    );

    let search_override = &config.rate_limiting.endpoint_overrides["search"];
    assert_eq!(search_override.requests_per_second, 5.0);
    assert_eq!(search_override.burst_capacity, 15);

    let issues_override = &config.rate_limiting.endpoint_overrides["issues"];
    assert_eq!(issues_override.requests_per_second, 20.0);
    assert_eq!(issues_override.burst_capacity, 50);

    clear_jira_env_vars();
}

#[test]
#[serial]
fn test_metrics_export_config() {
    clear_jira_env_vars();

    set_test_env_var("JIRA_METRICS_EXPORT_FORMAT", "prometheus");
    set_test_env_var(
        "JIRA_METRICS_EXPORT_ENDPOINT",
        "http://metrics.example.com/metrics",
    );
    set_test_env_var("JIRA_METRICS_EXPORT_INTERVAL", "60s");

    let config = load_config_from_env();

    assert_eq!(config.metrics.export.format, "prometheus");
    assert_eq!(
        config.metrics.export.endpoint,
        Some("http://metrics.example.com/metrics".to_string())
    );
    assert_eq!(config.metrics.export.interval, Duration::from_secs(60));

    clear_jira_env_vars();
}
