// Comprehensive error scenario testing for edge cases and failure modes
// These tests ensure robust error handling throughout the configuration system

use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

use gouqi::{Credentials, Error, FieldSchema, GouqiConfig, JiraBuilder};

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

fn clear_env_vars() {
    let vars = [
        "JIRA_HOST",
        "JIRA_URL",
        "JIRA_USER",
        "JIRA_PASS",
        "JIRA_TOKEN",
        "JIRA_COOKIE",
        "JIRA_TIMEOUT",
        "JIRA_MAX_RETRIES",
        "ERROR_TEST_VAR",
    ];
    for var in vars.iter() {
        remove_test_env_var(var);
    }
}

#[test]
fn test_builder_validation_errors() {
    clear_env_vars();

    // Test 1: Zero timeout
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .timeout(Duration::from_secs(0))
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message, .. } => assert!(message.contains("timeout")),
        _ => panic!("Expected ConfigError for timeout"),
    }

    // Test 2: Connect timeout greater than default timeout
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(15))
        .build_with_validation();

    assert!(result.is_err());
    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Connect timeout cannot be greater than default timeout"));

    // Test 3: Invalid retry configuration
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .retry_policy(0, Duration::from_millis(100), Duration::from_secs(10))
        .build_with_validation();

    assert!(result.is_err());
    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Max retry attempts must be greater than 0"));

    // Test 4: Max delay less than base delay
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .retry_policy(3, Duration::from_millis(1000), Duration::from_millis(500))
        .build_with_validation();

    assert!(result.is_err());
    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Max retry delay cannot be less than base delay"));
}

#[test]
fn test_credentials_validation_errors() {
    clear_env_vars();

    // Test 1: Empty username in Basic credentials
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("".to_string(), "password".to_string()))
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message } => assert!(message.contains("cannot be empty")),
        _ => panic!("Expected ConfigError for empty username"),
    }

    // Test 2: Empty password in Basic credentials
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "".to_string()))
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message } => assert!(message.contains("cannot be empty")),
        _ => panic!("Expected ConfigError for empty password"),
    }

    // Test 3: Empty Bearer token
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Bearer("".to_string()))
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message } => assert!(message.contains("cannot be empty")),
        _ => panic!("Expected ConfigError for empty bearer token"),
    }

    // Test 4: Empty Cookie
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Cookie("".to_string()))
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message } => assert!(message.contains("cannot be empty")),
        _ => panic!("Expected ConfigError for empty cookie"),
    }
}

#[test]
fn test_field_schema_validation_errors() {
    clear_env_vars();

    // Test 1: Enumeration field without allowed values
    let mut invalid_enum = FieldSchema::text(false);
    invalid_enum.field_type = "enum".to_string();
    invalid_enum.allowed_values = None; // No allowed values specified

    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .custom_field("invalid_enum", invalid_enum)
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::FieldSchemaError { field, message } => {
            assert_eq!(field, "invalid_enum");
            assert!(message.contains("Enum fields must specify allowed values"));
        }
        _ => panic!("Expected FieldSchemaError for enum without values"),
    }

    // Test 2: Number field with invalid range (min > max) - should panic
    let panic_result =
        std::panic::catch_unwind(|| FieldSchema::number(false, Some(100.0), Some(50.0)));
    assert!(panic_result.is_err());

    // Test 3: Array field without item_type
    let mut invalid_array = FieldSchema::text(false);
    invalid_array.field_type = "array".to_string();
    invalid_array.custom_properties.clear(); // No item_type specified

    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .custom_field("invalid_array", invalid_array)
        .build_with_validation();

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::FieldSchemaError { field, message } => {
            assert_eq!(field, "invalid_array");
            assert!(message.contains("Array fields must specify item_type"));
        }
        _ => panic!("Expected FieldSchemaError for array without item_type"),
    }

    // Test 4: Empty enumeration is allowed (but might not be practical)
    let result = FieldSchema::enumeration(true, Vec::<String>::new());
    assert!(result.is_ok());

    // Test 5: Enumeration with duplicate values should still work (but we can test other edge cases)
    let schema = FieldSchema::enumeration(true, vec!["High", "High", "Low"]);
    assert!(schema.is_ok()); // Duplicates are allowed
}

#[test]
fn test_configuration_file_errors() {
    clear_env_vars();

    // Test 1: Non-existent file
    let result = GouqiConfig::from_file("/non/existent/path.json");
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::IO(_) => {}
        _ => panic!("Expected IO error for non-existent file"),
    }

    // Test 2: Invalid JSON content
    let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
    temp_file.write_all(b"{ invalid json content }").unwrap();

    let result = GouqiConfig::from_file(temp_file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Serde(_) => {}
        _ => panic!("Expected Serde error for invalid JSON"),
    }

    // Test 3: Wrong file extension
    let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    temp_file.write_all(b"some content").unwrap();

    let result = GouqiConfig::from_file(temp_file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message, .. } => {
            assert!(message.contains("Unsupported config file format"));
        }
        _ => panic!("Expected ConfigError for wrong file extension"),
    }

    // Test 4: File without extension
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"{}").unwrap();

    let result = GouqiConfig::from_file(temp_file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::ConfigError { message, .. } => {
            assert!(message.contains("Config file must have an extension"));
        }
        _ => panic!("Expected ConfigError for missing file extension"),
    }

    // Test 5: Malformed YAML (if YAML feature is enabled)
    #[cfg(feature = "yaml")]
    {
        let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
        temp_file.write_all(b"invalid: yaml: content: [").unwrap();

        let result = GouqiConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }

    // Test 6: Malformed TOML (if TOML feature is enabled)
    #[cfg(feature = "toml-support")]
    {
        let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        temp_file.write_all(b"[invalid\ntoml content").unwrap();

        let result = GouqiConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}

#[test]
fn test_environment_variable_edge_cases() {
    clear_env_vars();

    // Test 1: Invalid boolean values should fall back to defaults
    set_test_env_var("JIRA_CACHE_ENABLED", "maybe");
    let config = gouqi::env::load_config_from_env();
    // Should use default (true) since "maybe" is not a valid boolean
    assert!(config.cache.enabled);

    // Test 2: Invalid duration formats should fall back to defaults
    set_test_env_var("JIRA_TIMEOUT", "invalid_duration");
    let config = gouqi::env::load_config_from_env();
    assert_eq!(config.timeout.default, Duration::from_secs(30)); // default

    // Test 3: Invalid numbers should fall back to defaults
    set_test_env_var("JIRA_MAX_RETRIES", "not_a_number");
    let config = gouqi::env::load_config_from_env();
    assert_eq!(config.retry.max_attempts, 3); // default

    // Test 4: Invalid floats should fall back to defaults
    set_test_env_var("JIRA_RATE_LIMIT_RPS", "not_a_float");
    let config = gouqi::env::load_config_from_env();
    assert_eq!(config.rate_limiting.requests_per_second, 10.0); // default

    // Test 5: Negative numbers where not expected
    set_test_env_var("JIRA_MAX_CONNECTIONS", "-5");
    let config = gouqi::env::load_config_from_env();
    // Should use default since negative doesn't make sense for connections
    assert_eq!(config.connection_pool.max_connections_per_host, 10); // default

    clear_env_vars();
}

#[test]
fn test_configuration_validation_edge_cases() {
    // Test 1: Configuration with all zero timeouts
    let mut config = GouqiConfig::default();
    config.timeout.default = Duration::from_secs(0);
    config.timeout.connect = Duration::from_secs(0);
    config.timeout.read = Duration::from_secs(0);

    let result = config.validate();
    assert!(result.is_err());

    // Test 2: Extremely large values
    let mut config = GouqiConfig::default();
    config.timeout.default = Duration::from_secs(u32::MAX as u64);
    config.connection_pool.max_connections_per_host = usize::MAX;
    config.cache.max_entries = usize::MAX;

    // These should be valid (though impractical)
    let result = config.validate();
    assert!(result.is_ok());

    // Test 3: Zero connection pool size
    let mut config = GouqiConfig::default();
    config.connection_pool.max_connections_per_host = 0;

    let result = config.validate();
    assert!(result.is_err());

    // Test 4: Zero cache entries
    let mut config = GouqiConfig::default();
    config.cache.max_entries = 0;
    config.cache.enabled = true;

    let result = config.validate();
    assert!(result.is_err());

    // Test 5: Invalid backoff multiplier
    let mut config = GouqiConfig::default();
    config.retry.backoff_multiplier = 0.0;

    let result = config.validate();
    assert!(result.is_err());

    // Test 6: Negative rate limiting values
    let mut config = GouqiConfig::default();
    config.rate_limiting.requests_per_second = -1.0;

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_concurrent_environment_access() {
    // Test that concurrent access to environment variables doesn't cause issues
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    clear_env_vars();

    // Set up some environment variables
    set_test_env_var("JIRA_TIMEOUT", "30");
    set_test_env_var("JIRA_MAX_RETRIES", "3");

    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn 5 threads that each load configuration 10 times
    for _ in 0..5 {
        let success_count_clone = success_count.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let config = gouqi::env::load_config_from_env();
                // Basic validation that the config was loaded successfully
                if config.timeout.default == Duration::from_secs(30)
                    && config.retry.max_attempts == 3
                {
                    success_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // All 50 loads should have been successful
    assert_eq!(success_count.load(Ordering::SeqCst), 50);

    clear_env_vars();
}

#[test]
fn test_builder_state_consistency() {
    // Test that builder state remains consistent even with conflicting operations

    let builder = JiraBuilder::new()
        .host("https://test1.atlassian.net")
        .host("https://test2.atlassian.net") // Override previous host
        .credentials(Credentials::Basic("user1".to_string(), "pass1".to_string()))
        .credentials(Credentials::Bearer("token".to_string())) // Override credentials
        .timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(40)); // Override timeout - must be >= read timeout

    // The final values should be the last ones set
    // We can't directly test this without getter methods, but we can test that build succeeds
    let result = builder
        .host("https://final.atlassian.net")
        .credentials(Credentials::Basic(
            "final_user".to_string(),
            "final_pass".to_string(),
        ))
        .build_with_validation();

    match &result {
        Ok(_) => {}
        Err(e) => println!("Builder state consistency test failed: {:?}", e),
    }
    assert!(result.is_ok());
}

#[test]
fn test_memory_pressure_scenarios() {
    // Test behavior under memory pressure scenarios

    // Create many configurations to test memory handling
    let configs: Result<Vec<_>, _> = (0..1000)
        .map(|i| {
            let mut custom_fields = HashMap::new();

            // Add several custom fields per configuration
            for j in 0..10 {
                custom_fields.insert(format!("field_{}_{}", i, j), FieldSchema::text(false));
            }

            JiraBuilder::new()
                .host(format!("https://test{}.atlassian.net", i))
                .credentials(Credentials::Basic(
                    format!("user{}", i),
                    format!("pass{}", i),
                ))
                .custom_fields(custom_fields)
                .build_with_validation()
        })
        .collect();

    // All configurations should build successfully
    assert!(configs.is_ok());
    let configs = configs.unwrap();
    assert_eq!(configs.len(), 1000);
}

#[test]
fn test_serialization_edge_cases() {
    // Test serialization/deserialization edge cases

    // Test 1: Config with extreme values
    let mut extreme_config = GouqiConfig::default();
    extreme_config.timeout.default = Duration::from_nanos(1); // Minimum duration
    extreme_config.connection_pool.max_connections_per_host = 1; // Minimum connections
    extreme_config.cache.max_entries = usize::MAX; // Maximum cache entries
    extreme_config.retry.backoff_multiplier = f64::MAX; // Maximum float

    let json = serde_json::to_string(&extreme_config).unwrap();
    let deserialized: GouqiConfig = serde_json::from_str(&json).unwrap();

    // Values should round-trip correctly
    assert_eq!(extreme_config.timeout.default, deserialized.timeout.default);
    assert_eq!(
        extreme_config.cache.max_entries,
        deserialized.cache.max_entries
    );

    // Test 2: Config with empty collections
    let mut empty_config = GouqiConfig::default();
    empty_config.cache.strategies.clear();
    empty_config.rate_limiting.endpoint_overrides.clear();

    let json = serde_json::to_string(&empty_config).unwrap();
    let deserialized: GouqiConfig = serde_json::from_str(&json).unwrap();

    assert!(deserialized.cache.strategies.is_empty());
    assert!(deserialized.rate_limiting.endpoint_overrides.is_empty());
}
