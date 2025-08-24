//! Advanced Configuration Example
//!
//! This example demonstrates the comprehensive configuration capabilities
//! of the gouqi Jira client, including:
//! - Enhanced builder pattern with validation
//! - Configuration from files (JSON, YAML, TOML)
//! - Environment variable loading
//! - Custom field schemas
//! - Configuration templates
//! - Advanced error handling

use std::collections::HashMap;
use std::time::Duration;
use std::{env, fs};

use gouqi::{ConfigTemplate, Credentials, FieldSchema, GouqiConfig, JiraBuilder, Result};

fn main() -> Result<()> {
    println!("=== Advanced Configuration Examples ===\n");

    // Example 1: Programmatic configuration with validation
    example_programmatic_configuration()?;

    // Example 2: Configuration from environment variables
    example_environment_configuration()?;

    // Example 3: Configuration from files
    example_file_configuration()?;

    // Example 4: Configuration templates
    example_configuration_templates()?;

    // Example 5: Custom field schemas
    example_custom_field_schemas()?;

    // Example 6: Advanced fluent configuration
    example_fluent_configuration()?;

    // Example 7: Error handling and validation
    example_error_handling()?;

    println!("All configuration examples completed successfully!");
    Ok(())
}

fn example_programmatic_configuration() -> Result<()> {
    println!("1. Programmatic Configuration:");

    let _jira = JiraBuilder::new()
        .host("https://company.atlassian.net")
        .credentials(Credentials::Basic(
            "user@company.com".to_string(),
            "api-token".to_string(),
        ))
        .timeout(Duration::from_secs(45))
        .retry_policy(5, Duration::from_millis(200), Duration::from_secs(30))
        .connection_pool_size(15)
        .memory_cache(Duration::from_secs(600), 2000)
        .rate_limit(15.0, 30)
        .user_agent("MyApp/1.0 (Rust gouqi)")
        .build_with_validation()?;

    println!("   ✓ Client created with comprehensive configuration");
    println!("   - Host: https://company.atlassian.net");
    println!("   - Timeout: 45 seconds");
    println!("   - Max retries: 5");
    println!("   - Connection pool: 15");
    println!("   - Cache TTL: 10 minutes, 2000 entries");
    println!("   - Rate limit: 15 RPS, burst 30");
    println!();

    Ok(())
}

fn example_environment_configuration() -> Result<()> {
    println!("2. Environment Variable Configuration:");

    // Set up example environment variables
    unsafe {
        env::set_var("JIRA_HOST", "https://env-test.atlassian.net");
    }
    unsafe {
        env::set_var("JIRA_USER", "env-user@company.com");
    }
    unsafe {
        env::set_var("JIRA_PASS", "env-api-token");
    }
    unsafe {
        env::set_var("JIRA_TIMEOUT", "60");
    }
    unsafe {
        env::set_var("JIRA_MAX_RETRIES", "3");
    }
    unsafe {
        env::set_var("JIRA_CONNECTION_POOL_SIZE", "20");
    }
    unsafe {
        env::set_var("JIRA_CACHE_TTL", "300s");
    }
    unsafe {
        env::set_var("JIRA_RATE_LIMIT_RPS", "25.0");
    }
    unsafe {
        env::set_var("JIRA_RATE_LIMIT_BURST", "50");
    }

    let _jira = JiraBuilder::new()
        .config_from_env()?
        .build_with_validation()?;

    println!("   ✓ Client created from environment variables");
    println!("   - JIRA_HOST: https://env-test.atlassian.net");
    println!("   - JIRA_TIMEOUT: 60 seconds");
    println!("   - JIRA_MAX_RETRIES: 3");
    println!("   - JIRA_CONNECTION_POOL_SIZE: 20");
    println!("   - JIRA_RATE_LIMIT_RPS: 25.0");

    // Cleanup
    let env_vars = [
        "JIRA_HOST",
        "JIRA_USER",
        "JIRA_PASS",
        "JIRA_TIMEOUT",
        "JIRA_MAX_RETRIES",
        "JIRA_CONNECTION_POOL_SIZE",
        "JIRA_CACHE_TTL",
        "JIRA_RATE_LIMIT_RPS",
        "JIRA_RATE_LIMIT_BURST",
    ];
    for var in env_vars.iter() {
        unsafe {
            env::remove_var(var);
        }
    }

    println!();
    Ok(())
}

fn example_file_configuration() -> Result<()> {
    println!("3. File-based Configuration:");

    // Create a sample JSON configuration
    let json_config = r#"
    {
        "timeout": {
            "default": "45s",
            "connect": "15s",
            "read": "45s"
        },
        "connection_pool": {
            "max_connections_per_host": 25,
            "idle_timeout": "60s",
            "http2": true,
            "keep_alive_timeout": "120s"
        },
        "cache": {
            "enabled": true,
            "default_ttl": "600s",
            "max_entries": 3000,
            "strategies": {
                "issues": {
                    "ttl": "300s",
                    "cache_errors": false,
                    "use_etag": true
                },
                "projects": {
                    "ttl": "1800s",
                    "cache_errors": true,
                    "use_etag": false
                }
            }
        },
        "metrics": {
            "enabled": true,
            "collection_interval": "120s",
            "collect_request_times": true,
            "collect_error_rates": true,
            "collect_cache_stats": true,
            "export": {
                "format": "prometheus",
                "endpoint": "http://metrics.company.com:9090/metrics",
                "interval": "300s"
            }
        },
        "retry": {
            "max_attempts": 5,
            "base_delay": "100ms",
            "max_delay": "30s",
            "backoff_multiplier": 2.0,
            "retry_status_codes": [429, 500, 502, 503, 504],
            "retry_on_connection_errors": true
        },
        "rate_limiting": {
            "enabled": true,
            "requests_per_second": 20.0,
            "burst_capacity": 50,
            "endpoint_overrides": {
                "search": {
                    "requests_per_second": 5.0,
                    "burst_capacity": 10
                },
                "issues/create": {
                    "requests_per_second": 2.0,
                    "burst_capacity": 5
                }
            }
        }
    }
    "#;

    // Write to temporary file
    fs::write("/tmp/jira_config_example.json", json_config)?;

    let _jira = JiraBuilder::new()
        .host("https://file-config.atlassian.net")
        .credentials(Credentials::Bearer("file-token".to_string()))
        .config_from_file("/tmp/jira_config_example.json")?
        .build_with_validation()?;

    println!("   ✓ Client created from JSON configuration file");
    println!("   - Timeout: 45 seconds");
    println!("   - Connection pool: 25 connections");
    println!("   - Cache: enabled, 3000 entries, 10 minutes TTL");
    println!("   - Metrics: enabled, Prometheus export");
    println!("   - Rate limiting: 20 RPS with endpoint overrides");

    // Cleanup
    let _ = fs::remove_file("/tmp/jira_config_example.json");

    println!();
    Ok(())
}

fn example_configuration_templates() -> Result<()> {
    println!("4. Configuration Templates:");

    // High throughput configuration
    let _high_throughput_jira = JiraBuilder::new()
        .host("https://high-throughput.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
        .config_template(ConfigTemplate::HighThroughput)
        .build_with_validation()?;

    println!("   ✓ High Throughput Template:");
    println!("   - Optimized for high-volume operations");
    println!("   - 50 connections per host");
    println!("   - 50 RPS rate limit with 100 burst capacity");
    println!("   - 5000 cache entries");

    // Low resource configuration
    let _low_resource_jira = JiraBuilder::new()
        .host("https://low-resource.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
        .config_template(ConfigTemplate::LowResource)
        .build_with_validation()?;

    println!("   ✓ Low Resource Template:");
    println!("   - Optimized for resource-constrained environments");
    println!("   - 2 connections per host");
    println!("   - 2 RPS rate limit with 5 burst capacity");
    println!("   - 100 cache entries");
    println!("   - Metrics disabled");

    println!();
    Ok(())
}

fn example_custom_field_schemas() -> Result<()> {
    println!("5. Custom Field Schemas:");

    // Define various custom field schemas
    let story_points = FieldSchema::number(false, Some(0.0), Some(100.0)).with_default(0)?;

    let priority =
        FieldSchema::enumeration(true, vec!["Highest", "High", "Medium", "Low", "Lowest"])?;

    let epic_link = FieldSchema::text(false).with_property("max_length", 255)?;

    let sprint_start_date = FieldSchema::datetime(false);

    let components = FieldSchema::array(false, "string");

    let is_flagged = FieldSchema::boolean(false, Some(false));

    let custom_number_field = FieldSchema::integer(true, Some(1), Some(1000))
        .with_property("step", 1)?
        .with_property("validation_rule", "must be positive")?;

    // Create a comprehensive custom fields map
    let mut custom_fields = HashMap::new();
    custom_fields.insert("customfield_10001".to_string(), story_points);
    custom_fields.insert("customfield_10002".to_string(), priority);
    custom_fields.insert("customfield_10003".to_string(), epic_link);
    custom_fields.insert("customfield_10004".to_string(), sprint_start_date);
    custom_fields.insert("customfield_10005".to_string(), components);
    custom_fields.insert("customfield_10006".to_string(), is_flagged);
    custom_fields.insert("customfield_10007".to_string(), custom_number_field);

    let _jira = JiraBuilder::new()
        .host("https://custom-fields.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
        .custom_fields(custom_fields)
        .build_with_validation()?;

    println!("   ✓ Custom Field Schemas defined:");
    println!("   - Story Points: number (0-100) with default 0");
    println!("   - Priority: enumeration (5 levels) - required");
    println!("   - Epic Link: text with max length validation");
    println!("   - Sprint Start Date: datetime - optional");
    println!("   - Components: array of strings");
    println!("   - Is Flagged: boolean with default false");
    println!("   - Custom Number: integer (1-1000) with validation");

    println!();
    Ok(())
}

fn example_fluent_configuration() -> Result<()> {
    println!("6. Advanced Fluent Configuration:");

    let _jira = JiraBuilder::new()
        .host("https://advanced.atlassian.net")
        .credentials(Credentials::Bearer("advanced-token".to_string()))
        // Timeout configuration
        .timeout(Duration::from_secs(90))
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(75))
        // Retry configuration
        .retry_policy(7, Duration::from_millis(150), Duration::from_secs(45))
        .retry_backoff(1.8)
        .retry_status_codes(vec![408, 429, 500, 502, 503, 504])
        // Connection pool configuration
        .connection_pool(40, Duration::from_secs(90), true)
        // Security configuration
        .validate_ssl(true)
        .user_agent("AdvancedJiraClient/2.0 (+https://company.com/jira-client)")
        // Caching configuration
        .memory_cache(Duration::from_secs(900), 5000) // 15 minutes, 5000 entries
        // Rate limiting configuration
        .rate_limit(30.0, 75)
        // Metrics configuration
        .enable_metrics()
        .metrics_config(Duration::from_secs(30), "prometheus")
        .build_with_validation()?;

    println!("   ✓ Advanced fluent configuration created:");
    println!("   - Extended timeouts (90s default, 15s connect, 75s read)");
    println!("   - Aggressive retry policy (7 attempts, 1.8x backoff)");
    println!("   - Large connection pool (40 connections)");
    println!("   - Extended cache (15 min TTL, 5000 entries)");
    println!("   - Higher rate limits (30 RPS, 75 burst)");
    println!("   - Prometheus metrics (30s collection interval)");

    println!();
    Ok(())
}

fn example_error_handling() -> Result<()> {
    println!("7. Error Handling and Validation:");

    // Example 1: Invalid timeout configuration
    println!("   Testing invalid timeout configuration...");
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .timeout(Duration::from_secs(0)) // Invalid - zero timeout
        .build_with_validation();

    match result {
        Err(e) => println!("   ✓ Caught expected error: {}", e),
        Ok(_) => println!("   ✗ Expected error but validation passed"),
    }

    // Example 2: Invalid custom field schema
    println!("   Testing invalid custom field schema...");
    let mut invalid_enum = FieldSchema::text(false);
    invalid_enum.field_type = "enum".to_string(); // Set as enum but no allowed values

    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .custom_field("invalid_enum", invalid_enum)
        .build_with_validation();

    match result {
        Err(e) => println!("   ✓ Caught expected error: {}", e),
        Ok(_) => println!("   ✗ Expected error but validation passed"),
    }

    // Example 3: Invalid credentials
    println!("   Testing empty credentials...");
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("".to_string(), "pass".to_string())) // Empty username
        .build_with_validation();

    match result {
        Err(e) => println!("   ✓ Caught expected error: {}", e),
        Ok(_) => println!("   ✗ Expected error but validation passed"),
    }

    // Example 4: Configuration validation through config object
    println!("   Testing configuration validation...");
    let mut config = GouqiConfig::default();
    config.retry.max_delay = Duration::from_millis(50);
    config.retry.base_delay = Duration::from_millis(100); // max < base - invalid

    match config.validate() {
        Err(e) => println!("   ✓ Caught expected config error: {}", e),
        Ok(_) => println!("   ✗ Expected config validation error"),
    }

    // Example 5: Invalid file configuration
    println!("   Testing invalid file configuration...");
    fs::write("/tmp/invalid_config.json", "{ invalid json content }")?;

    let result = JiraBuilder::new().config_from_file("/tmp/invalid_config.json");

    match result {
        Err(e) => println!("   ✓ Caught expected file error: {}", e),
        Ok(_) => println!("   ✗ Expected file parsing error"),
    }

    // Cleanup
    let _ = fs::remove_file("/tmp/invalid_config.json");

    println!();
    Ok(())
}
