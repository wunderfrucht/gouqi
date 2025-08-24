use gouqi::{ConfigTemplate, Credentials, FieldSchema, JiraBuilder};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

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

#[test]
fn test_builder_basic_usage() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .timeout(Duration::from_secs(60))
        .build();

    assert!(result.is_ok());
}

#[test]
#[should_panic(expected = "Invalid host URL:")]
fn test_builder_invalid_host_url() {
    JiraBuilder::new().host("not-a-valid-url");
}

#[test]
#[should_panic(expected = "Invalid host URL:")]
fn test_builder_validation_invalid_url() {
    // This test verifies that invalid URLs cause panics at the builder level
    JiraBuilder::new()
        .host("invalid-url") // This should panic immediately
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()));
}

#[test]
fn test_builder_credentials_validation() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("".to_string(), "pass".to_string()))
        .build_with_validation();

    assert!(result.is_err());
}

#[test]
fn test_builder_empty_password_validation() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "".to_string()))
        .build_with_validation();

    assert!(result.is_err());
}

#[test]
fn test_builder_bearer_token_validation() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Bearer("".to_string()))
        .build_with_validation();

    assert!(result.is_err());
}

#[test]
fn test_builder_from_env() {
    // Set up environment variables
    set_test_env_var("JIRA_HOST", "https://env-test.atlassian.net");
    set_test_env_var("JIRA_USER", "env-user");
    set_test_env_var("JIRA_PASS", "env-pass");

    let result = JiraBuilder::new().config_from_env();

    // Just verify that the builder creation succeeds
    assert!(result.is_ok());

    // Cleanup
    remove_test_env_var("JIRA_HOST");
    remove_test_env_var("JIRA_USER");
    remove_test_env_var("JIRA_PASS");
}

#[test]
fn test_config_template_high_throughput() {
    let result = JiraBuilder::new()
        .host("https://high-throughput.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
        .config_template(ConfigTemplate::HighThroughput)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_config_template_low_resource() {
    let result = JiraBuilder::new()
        .host("https://low-resource.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
        .config_template(ConfigTemplate::LowResource)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_fluent_api_comprehensive() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Bearer("token".to_string()))
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(100))
        .retry_policy(5, Duration::from_millis(200), Duration::from_secs(60))
        .retry_backoff(1.8)
        .retry_status_codes(vec![429, 500, 503])
        .connection_pool(30, Duration::from_secs(45), false)
        .validate_ssl(true)
        .user_agent("TestAgent/1.0")
        .enable_cache()
        .memory_cache(Duration::from_secs(600), 2000)
        .rate_limit(20.0, 40)
        .enable_metrics()
        .metrics_config(Duration::from_secs(120), "prometheus")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_cache_configuration() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .enable_cache()
        .memory_cache(Duration::from_secs(300), 1500)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_disable_cache_configuration() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .disable_cache()
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_rate_limiting_configuration() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .rate_limit(15.5, 35)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_disable_rate_limiting() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .disable_rate_limiting()
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_metrics_configuration() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .enable_metrics()
        .metrics_config(Duration::from_secs(90), "json")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_disable_metrics() {
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .disable_metrics()
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_custom_fields_basic() {
    let story_points = FieldSchema::number(false, Some(0.0), Some(100.0));
    let priority = FieldSchema::enumeration(true, vec!["High", "Medium", "Low"]).unwrap();
    let epic_link = FieldSchema::text(false);

    let mut custom_fields = HashMap::new();
    custom_fields.insert("story_points".to_string(), story_points);
    custom_fields.insert("priority".to_string(), priority);
    custom_fields.insert("epic_link".to_string(), epic_link);

    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .custom_fields(custom_fields)
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_field_schema_validation_invalid_enum() {
    let mut invalid_enum = FieldSchema::text(false);
    invalid_enum.field_type = "enum".to_string(); // Set as enum but no allowed values

    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .custom_field("invalid_enum", invalid_enum)
        .build_with_validation();

    assert!(result.is_err());
}
