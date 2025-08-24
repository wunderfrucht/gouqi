// Real-world usage validation tests
// These tests simulate actual patterns and workflows that users would employ
// with the Jira client configuration system in production environments

use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tempfile::NamedTempFile;

use gouqi::{ConfigTemplate, Credentials, FieldSchema, GouqiConfig, JiraBuilder};

// Helper functions for safe environment variable operations
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
    let vars = [
        "JIRA_HOST",
        "JIRA_URL",
        "JIRA_USER",
        "JIRA_PASS",
        "JIRA_TOKEN",
        "JIRA_TIMEOUT",
        "JIRA_MAX_RETRIES",
        "JIRA_CACHE_ENABLED",
        "JIRA_METRICS_ENABLED",
        "JIRA_RATE_LIMITING_ENABLED",
    ];
    for var in vars.iter() {
        remove_test_env_var(var);
    }
}

#[test]
fn test_typical_development_workflow() {
    // Simulate a typical development setup where a developer
    // configures the client for local development
    clear_jira_env_vars();

    let client = JiraBuilder::new()
        .host("https://mycompany.atlassian.net")
        .credentials(Credentials::Basic(
            "developer@company.com".to_string(),
            "api-token-here".to_string(),
        ))
        .timeout(Duration::from_secs(30))
        .retry_policy(3, Duration::from_millis(500), Duration::from_secs(10))
        .memory_cache(Duration::from_secs(300), 100) // Small cache for development
        .enable_metrics()
        .user_agent("MyApp-Development/1.0")
        .build_with_validation();

    assert!(
        client.is_ok(),
        "Development workflow configuration should be valid"
    );
}

#[test]
fn test_production_high_load_setup() {
    // Simulate a production setup for high-throughput applications
    clear_jira_env_vars();

    let client = JiraBuilder::new()
        .host("https://enterprise.atlassian.net")
        .credentials(Credentials::Bearer("production-jwt-token".to_string()))
        .config_template(ConfigTemplate::HighThroughput)
        // Override some specific settings
        .connection_pool(50, Duration::from_secs(120), true)
        .rate_limit(20.0, 50) // Higher rate limit for production
        .memory_cache(Duration::from_secs(600), 10000) // Large cache
        .retry_policy(5, Duration::from_millis(200), Duration::from_secs(30))
        .enable_metrics()
        .user_agent("Enterprise-App/2.1.0")
        .build_with_validation();

    assert!(
        client.is_ok(),
        "Production high-load configuration should be valid"
    );
}

#[test]
fn test_resource_constrained_environment() {
    // Simulate IoT or embedded device with limited resources
    clear_jira_env_vars();

    let client = JiraBuilder::new()
        .host("https://iot-backend.atlassian.net")
        .credentials(Credentials::Basic(
            "iot-device".to_string(),
            "device-token".to_string(),
        ))
        .config_template(ConfigTemplate::LowResource)
        // Further constrain resources
        .connection_pool(2, Duration::from_secs(30), false) // Minimal connections
        .memory_cache(Duration::from_secs(60), 50) // Very small cache
        .timeout(Duration::from_secs(15)) // Shorter timeouts
        .retry_policy(2, Duration::from_millis(1000), Duration::from_secs(5)) // Fewer retries
        .disable_metrics() // Save memory
        .user_agent("IoT-Device/0.8")
        .build_with_validation();

    assert!(
        client.is_ok(),
        "Resource-constrained configuration should be valid"
    );
}

#[test]
fn test_environment_based_configuration() {
    // Simulate 12-factor app configuration through environment variables
    clear_jira_env_vars();

    // Set environment variables as they would be in production
    set_test_env_var("JIRA_HOST", "https://prod.atlassian.net");
    set_test_env_var("JIRA_USER", "service-account");
    set_test_env_var("JIRA_PASS", "encrypted-password");
    set_test_env_var("JIRA_TIMEOUT", "45");
    set_test_env_var("JIRA_MAX_RETRIES", "5");
    set_test_env_var("JIRA_MAX_CONNECTIONS", "25");
    set_test_env_var("JIRA_CACHE_ENABLED", "true");
    set_test_env_var("JIRA_CACHE_TTL", "600");
    set_test_env_var("JIRA_METRICS_ENABLED", "true");
    set_test_env_var("JIRA_RATE_LIMITING_ENABLED", "true");
    set_test_env_var("JIRA_RATE_LIMIT_RPS", "15.0");

    // Load configuration from environment
    let client = JiraBuilder::new()
        .host("https://prod.atlassian.net") // Host still needs to be set manually
        .credentials(Credentials::Basic(
            "service-account".to_string(),
            "encrypted-password".to_string(),
        ))
        .config_from_env()
        .expect("Should load config from environment")
        .build_with_validation();

    match &client {
        Ok(_) => {}
        Err(e) => println!("Environment config failed: {:?}", e),
    }
    assert!(
        client.is_ok(),
        "Environment-based configuration should be valid"
    );

    // Cleanup
    clear_jira_env_vars();
}

#[test]
fn test_configuration_file_workflow() {
    // Simulate loading configuration from various file formats
    let config = GouqiConfig::high_throughput();

    // Test JSON file workflow
    let json_file = NamedTempFile::with_suffix(".json").unwrap();
    config
        .save_to_file(json_file.path())
        .expect("Should save JSON config");

    let loaded_config = GouqiConfig::from_file(json_file.path()).expect("Should load JSON config");
    assert_eq!(config.timeout.default, loaded_config.timeout.default);

    // Use loaded config with builder by loading from file directly
    let client = JiraBuilder::new()
        .host("https://file-config.atlassian.net")
        .credentials(Credentials::Bearer("file-token".to_string()))
        .config_from_file(json_file.path())
        .expect("Should load config from file")
        .build_with_validation();

    assert!(client.is_ok(), "File-based configuration should be valid");
}

#[test]
fn test_custom_field_heavy_workflow() {
    // Simulate an application that works with many custom fields
    // (common in enterprise Jira setups)
    clear_jira_env_vars();

    let mut custom_fields = HashMap::new();

    // Story tracking fields
    custom_fields.insert(
        "story_points".to_string(),
        FieldSchema::number(false, Some(0.0), Some(100.0)),
    );
    custom_fields.insert("epic_link".to_string(), FieldSchema::text(false));

    // Priority and categorization
    custom_fields.insert(
        "business_priority".to_string(),
        FieldSchema::enumeration(true, vec!["Critical", "High", "Medium", "Low", "Deferred"])
            .unwrap(),
    );
    custom_fields.insert(
        "customer_impact".to_string(),
        FieldSchema::enumeration(false, vec!["None", "Minor", "Major", "Severe"]).unwrap(),
    );

    // Metadata fields
    custom_fields.insert(
        "time_estimate_hours".to_string(),
        FieldSchema::number(false, Some(0.0), Some(1000.0)),
    );
    custom_fields.insert(
        "requires_testing".to_string(),
        FieldSchema::boolean(false, Some(true)),
    );
    custom_fields.insert("due_date".to_string(), FieldSchema::date(false));

    // Team and component fields
    custom_fields.insert(
        "assigned_team".to_string(),
        FieldSchema::enumeration(false, vec!["Frontend", "Backend", "DevOps", "QA", "Design"])
            .unwrap(),
    );
    custom_fields.insert(
        "affected_components".to_string(),
        FieldSchema::array(false, "string"),
    );

    // Build client with extensive custom fields
    let client = JiraBuilder::new()
        .host("https://enterprise-custom.atlassian.net")
        .credentials(Credentials::Basic(
            "project-manager".to_string(),
            "pm-token".to_string(),
        ))
        .custom_fields(custom_fields)
        .timeout(Duration::from_secs(60)) // Longer timeout for complex operations
        .memory_cache(Duration::from_secs(1200), 5000) // Large cache for field definitions
        .enable_metrics()
        .build_with_validation();

    assert!(
        client.is_ok(),
        "Custom field heavy configuration should be valid"
    );
}

#[test]
fn test_multi_environment_configuration_patterns() {
    // Simulate patterns where the same code runs in dev/staging/prod
    // with different configurations
    clear_jira_env_vars();

    // Development environment
    let dev_client = JiraBuilder::new()
        .host("https://dev.atlassian.net")
        .credentials(Credentials::Basic(
            "dev-user".to_string(),
            "dev-token".to_string(),
        ))
        .timeout(Duration::from_secs(35)) // Must be >= read timeout
        .read_timeout(Duration::from_secs(10)) // Fast read timeout for dev
        .retry_policy(1, Duration::from_millis(100), Duration::from_secs(2)) // Minimal retries
        .connection_pool(5, Duration::from_secs(30), false)
        .disable_metrics() // No metrics in dev
        .user_agent("MyApp-Development")
        .build_with_validation();
    match &dev_client {
        Ok(_) => {}
        Err(e) => println!("Dev client failed: {:?}", e),
    }
    assert!(dev_client.is_ok());

    // Staging environment
    let staging_client = JiraBuilder::new()
        .host("https://staging.atlassian.net")
        .credentials(Credentials::Bearer("staging-jwt".to_string()))
        .timeout(Duration::from_secs(30))
        .retry_policy(3, Duration::from_millis(300), Duration::from_secs(10))
        .connection_pool(10, Duration::from_secs(60), true)
        .memory_cache(Duration::from_secs(300), 500)
        .enable_metrics() // Enable metrics in staging
        .user_agent("MyApp-Staging")
        .build_with_validation();
    assert!(staging_client.is_ok());

    // Production environment
    let prod_client = JiraBuilder::new()
        .host("https://prod.atlassian.net")
        .credentials(Credentials::Bearer("prod-jwt-secure".to_string()))
        .config_template(ConfigTemplate::HighThroughput)
        .timeout(Duration::from_secs(60)) // Generous timeout for prod
        .retry_policy(5, Duration::from_millis(500), Duration::from_secs(30))
        .connection_pool(50, Duration::from_secs(120), true)
        .memory_cache(Duration::from_secs(3600), 10000) // Large, long-lived cache
        .rate_limit(25.0, 100) // High rate limits for prod
        .enable_metrics()
        .user_agent("MyApp-Production/1.2.5")
        .build_with_validation();
    assert!(prod_client.is_ok());
}

#[test]
fn test_microservice_configuration_patterns() {
    // Simulate configuration patterns common in microservice architectures
    clear_jira_env_vars();

    // User service - needs basic issue operations
    let user_service = JiraBuilder::new()
        .host("https://services.atlassian.net")
        .credentials(Credentials::Bearer("user-service-token".to_string()))
        .timeout(Duration::from_secs(35)) // Must be >= read timeout
        .read_timeout(Duration::from_secs(15)) // Set explicit read timeout
        .retry_policy(3, Duration::from_millis(200), Duration::from_secs(5))
        .connection_pool(8, Duration::from_secs(60), true)
        .rate_limit(10.0, 20) // Conservative rate limiting
        .user_agent("UserService/1.0")
        .build_with_validation();
    assert!(user_service.is_ok());

    // Notification service - batch operations
    let notification_service = JiraBuilder::new()
        .host("https://services.atlassian.net")
        .credentials(Credentials::Bearer(
            "notification-service-token".to_string(),
        ))
        .timeout(Duration::from_secs(45)) // Longer timeout for batch operations
        .retry_policy(5, Duration::from_millis(1000), Duration::from_secs(20))
        .connection_pool(3, Duration::from_secs(30), false) // Fewer connections
        .memory_cache(Duration::from_secs(1800), 2000) // Cache for user lookups
        .rate_limit(5.0, 10) // Lower rate limit for background service
        .user_agent("NotificationService/2.1")
        .build_with_validation();
    assert!(notification_service.is_ok());

    // Analytics service - read-heavy operations
    let analytics_service = JiraBuilder::new()
        .host("https://services.atlassian.net")
        .credentials(Credentials::Bearer("analytics-service-token".to_string()))
        .timeout(Duration::from_secs(120)) // Very long timeout for complex queries
        .retry_policy(2, Duration::from_millis(2000), Duration::from_secs(10))
        .connection_pool(15, Duration::from_secs(180), true) // Many concurrent queries
        .memory_cache(Duration::from_secs(3600), 15000) // Large cache for query results
        .rate_limit(3.0, 5) // Very conservative rate limiting
        .enable_metrics() // Detailed metrics for analytics
        .user_agent("AnalyticsService/1.5")
        .build_with_validation();
    assert!(analytics_service.is_ok());
}

#[test]
fn test_configuration_override_patterns() {
    // Test common patterns where base configuration is overridden
    clear_jira_env_vars();

    // Test high-priority configuration pattern
    let priority_client = JiraBuilder::new()
        .host("https://priority.atlassian.net")
        .credentials(Credentials::Bearer("priority-token".to_string()))
        .config_template(ConfigTemplate::HighThroughput)
        .timeout(Duration::from_secs(60)) // Override the template timeout (must be >= read timeout)
        .build_with_validation();
    match &priority_client {
        Ok(_) => {}
        Err(e) => println!("Priority client failed: {:?}", e),
    }
    assert!(priority_client.is_ok());

    // Test resource-constrained configuration pattern
    let constrained_client = JiraBuilder::new()
        .host("https://constrained.atlassian.net")
        .credentials(Credentials::Basic(
            "limited-user".to_string(),
            "limited-token".to_string(),
        ))
        .config_template(ConfigTemplate::LowResource)
        .connection_pool(1, Duration::from_secs(10), false) // Even more constrained
        .build_with_validation();
    assert!(constrained_client.is_ok());
}

#[test]
fn test_error_recovery_patterns() {
    // Test configuration patterns designed for robust error recovery
    clear_jira_env_vars();

    // Configuration optimized for unreliable networks
    let unreliable_network_client = JiraBuilder::new()
        .host("https://unreliable.atlassian.net")
        .credentials(Credentials::Bearer("network-token".to_string()))
        .timeout(Duration::from_secs(90)) // Long timeout for slow networks
        .connect_timeout(Duration::from_secs(30)) // Separate connection timeout
        .read_timeout(Duration::from_secs(60)) // Separate read timeout
        .retry_policy(7, Duration::from_millis(1000), Duration::from_secs(60)) // Aggressive retries
        .retry_backoff(2.5) // Exponential backoff
        .connection_pool(5, Duration::from_secs(300), false) // Keep connections alive longer
        .user_agent("RobustClient/1.0")
        .build_with_validation();
    assert!(unreliable_network_client.is_ok());

    // Configuration for API rate limit management
    let rate_limited_client = JiraBuilder::new()
        .host("https://rate-limited.atlassian.net")
        .credentials(Credentials::Bearer("rate-token".to_string()))
        .timeout(Duration::from_secs(45))
        .retry_policy(10, Duration::from_millis(2000), Duration::from_secs(120)) // Many retries with long delays
        .rate_limit(2.0, 5) // Very conservative rate limiting
        .memory_cache(Duration::from_secs(7200), 20000) // Large cache to reduce API calls
        .user_agent("ConservativeClient/1.0")
        .build_with_validation();
    assert!(rate_limited_client.is_ok());
}

#[test]
fn test_monitoring_and_observability_patterns() {
    // Test configuration patterns for monitoring and observability
    clear_jira_env_vars();

    // Set up monitoring environment variables
    set_test_env_var("JIRA_METRICS_ENABLED", "true");
    set_test_env_var("JIRA_METRICS_COLLECT_REQUEST_TIMES", "true");
    set_test_env_var("JIRA_METRICS_COLLECT_ERROR_RATES", "true");
    set_test_env_var("JIRA_METRICS_COLLECT_CACHE_STATS", "true");
    set_test_env_var("JIRA_METRICS_EXPORT_FORMAT", "prometheus");
    set_test_env_var("JIRA_METRICS_EXPORT_INTERVAL", "30");

    // Production monitoring configuration
    let monitored_client = JiraBuilder::new()
        .host("https://monitored.atlassian.net")
        .credentials(Credentials::Bearer("monitoring-token".to_string()))
        .config_from_env()
        .expect("Should load monitoring config from env")
        .timeout(Duration::from_secs(45))
        .enable_metrics() // Ensure metrics are enabled
        .memory_cache(Duration::from_secs(900), 5000) // Cache for metrics collection
        .user_agent("MonitoredApp/2.0 (+monitoring)")
        .build_with_validation();
    assert!(monitored_client.is_ok());

    // Cleanup
    clear_jira_env_vars();
}

#[test]
fn test_security_conscious_patterns() {
    // Test configuration patterns for security-conscious environments
    clear_jira_env_vars();

    // Minimal privileges configuration
    let secure_client = JiraBuilder::new()
        .host("https://secure.atlassian.net")
        .credentials(Credentials::Bearer("limited-scope-jwt".to_string()))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10)) // Quick connection timeout
        .retry_policy(3, Duration::from_millis(500), Duration::from_secs(10))
        .connection_pool(5, Duration::from_secs(60), false) // No HTTP/2 for compatibility
        .disable_metrics() // Don't collect potentially sensitive metrics
        .memory_cache(Duration::from_secs(300), 100) // Small cache to limit data retention
        .user_agent("SecureApp/1.0") // Minimal user agent
        .build_with_validation();
    assert!(secure_client.is_ok());

    // Zero-cache configuration for highly sensitive environments
    let zero_cache_client = JiraBuilder::new()
        .host("https://zero-cache.atlassian.net")
        .credentials(Credentials::Bearer("ephemeral-token".to_string()))
        .timeout(Duration::from_secs(35)) // Must be >= read timeout
        .read_timeout(Duration::from_secs(15)) // Set explicit read timeout
        .retry_policy(1, Duration::from_millis(100), Duration::from_secs(2)) // Minimal retries
        .connection_pool(1, Duration::from_secs(10), false) // Single connection
        .disable_metrics()
        .disable_cache() // Disable caching completely
        .user_agent("EphemeralClient/1.0")
        .build_with_validation();
    match &zero_cache_client {
        Ok(_) => {}
        Err(e) => println!("Zero cache client failed: {:?}", e),
    }
    assert!(zero_cache_client.is_ok());
}

#[test]
fn test_configuration_validation_in_realistic_scenarios() {
    // Test that validation catches real-world configuration mistakes
    clear_jira_env_vars();

    // Common mistake: timeout too short for retry policy
    let invalid_timeout_result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Bearer("test-token".to_string()))
        .timeout(Duration::from_secs(5)) // Short timeout
        .retry_policy(5, Duration::from_millis(2000), Duration::from_secs(30)) // Long retry delays
        .build_with_validation();
    assert!(
        invalid_timeout_result.is_err(),
        "Should reject timeout shorter than retry max delay"
    );

    // Common mistake: read timeout greater than default timeout
    let invalid_read_timeout_result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Bearer("test-token".to_string()))
        .timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(30)) // Read timeout > default timeout
        .build_with_validation();
    assert!(
        invalid_read_timeout_result.is_err(),
        "Should reject read timeout greater than default timeout"
    );

    // Common mistake: empty credentials
    let invalid_creds_result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("".to_string(), "password".to_string()))
        .build_with_validation();
    assert!(
        invalid_creds_result.is_err(),
        "Should reject empty username"
    );

    // Valid configuration should pass
    let valid_result = JiraBuilder::new()
        .host("https://valid.atlassian.net")
        .credentials(Credentials::Bearer("valid-token".to_string()))
        .timeout(Duration::from_secs(60))
        .read_timeout(Duration::from_secs(45))
        .retry_policy(3, Duration::from_millis(500), Duration::from_secs(10))
        .build_with_validation();
    assert!(
        valid_result.is_ok(),
        "Valid configuration should pass validation"
    );
}
