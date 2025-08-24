// Integration tests for configuration with real Jira instances
// These tests require environment variables to be set for actual API testing
// If not set, they will be skipped with appropriate warnings

use std::env;
use std::time::Duration;

use gouqi::{Credentials, GouqiConfig, JiraBuilder};
use serial_test::serial;

// Helper to check if integration tests should run
fn should_run_integration_tests() -> Option<(String, Credentials)> {
    let host = env::var("INTEGRATION_JIRA_HOST").ok()?;

    // Try different credential patterns
    if let Ok(token) = env::var("INTEGRATION_JIRA_TOKEN") {
        if !token.is_empty() {
            return Some((host, Credentials::Bearer(token)));
        }
    }

    if let (Ok(user), Ok(pass)) = (
        env::var("INTEGRATION_JIRA_USER"),
        env::var("INTEGRATION_JIRA_PASS"),
    ) {
        if !user.is_empty() && !pass.is_empty() {
            return Some((host, Credentials::Basic(user, pass)));
        }
    }

    None
}

// Macro to skip integration tests if credentials not available
macro_rules! integration_test {
    ($name:ident, $test_fn:expr) => {
        #[test]
        fn $name() {
            match should_run_integration_tests() {
                Some((host, creds)) => {
                    println!("Running integration test against: {}", host);
                    $test_fn(host, creds).expect("Integration test failed");
                }
                None => {
                    println!("Skipping {} - set INTEGRATION_JIRA_HOST and credentials", stringify!($name));
                    println!("  Credentials: INTEGRATION_JIRA_TOKEN or (INTEGRATION_JIRA_USER + INTEGRATION_JIRA_PASS)");
                }
            }
        }
    };
}

integration_test!(
    test_basic_configuration_integration,
    |host: String, creds: Credentials| -> Result<(), Box<dyn std::error::Error>> {
        // Test basic configuration with real API
        let _client = JiraBuilder::new()
            .host(host)
            .credentials(creds)
            .timeout(Duration::from_secs(30))
            .build_with_validation()?;

        // This should not panic and should establish a connection
        println!("✓ Basic configuration client created successfully");
        Ok(())
    }
);

#[test]
#[serial]
fn test_environment_config_integration() {
    match should_run_integration_tests() {
        Some((host, creds)) => {
            println!("Running integration test against: {}", host);
            let test_fn = |_host: String, _creds: Credentials| -> Result<(), Box<dyn std::error::Error>> {
        // Test environment-based configuration
        // Note: We don't override the provided credentials since the macro already verified they work

        // Set some additional config via environment
        unsafe {
            env::set_var("JIRA_TIMEOUT", "45");
            env::set_var("JIRA_MAX_RETRIES", "5");
            env::set_var("JIRA_CACHE_ENABLED", "true");
            env::set_var("JIRA_METRICS_ENABLED", "false"); // Disable metrics for integration testing
        }

        let _client = JiraBuilder::new()
            .config_from_env()?
            .build_with_validation()?;

        println!("✓ Environment configuration client created successfully");

        // Cleanup
        unsafe {
            env::remove_var("JIRA_TIMEOUT");
            env::remove_var("JIRA_MAX_RETRIES");
            env::remove_var("JIRA_CACHE_ENABLED");
            env::remove_var("JIRA_METRICS_ENABLED");
        }

        Ok(())
    };
            test_fn(host, creds).expect("Integration test failed");
        }
        None => {
            println!("Skipping test_environment_config_integration - set INTEGRATION_JIRA_HOST and credentials");
            println!("  Credentials: INTEGRATION_JIRA_TOKEN or (INTEGRATION_JIRA_USER + INTEGRATION_JIRA_PASS)");
        }
    }
}

integration_test!(
    test_high_throughput_config_integration,
    |host: String, creds: Credentials| -> Result<(), Box<dyn std::error::Error>> {
        // Test high throughput configuration template
        let _client = JiraBuilder::new()
            .host(host)
            .credentials(creds)
            .config_template(gouqi::ConfigTemplate::HighThroughput)
            .build_with_validation()?;

        println!("✓ High throughput configuration client created successfully");
        Ok(())
    }
);

integration_test!(
    test_low_resource_config_integration,
    |host: String, creds: Credentials| -> Result<(), Box<dyn std::error::Error>> {
        // Test low resource configuration template
        let _client = JiraBuilder::new()
            .host(host)
            .credentials(creds)
            .config_template(gouqi::ConfigTemplate::LowResource)
            .build_with_validation()?;

        println!("✓ Low resource configuration client created successfully");
        Ok(())
    }
);

integration_test!(test_custom_timeouts_integration, |host: String,
                                                     creds: Credentials|
 -> Result<
    (),
    Box<dyn std::error::Error>,
> {
    // Test custom timeout configurations
    let _client = JiraBuilder::new()
        .host(host)
        .credentials(creds)
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(45))
        .build_with_validation()?;

    println!("✓ Custom timeout configuration client created successfully");
    Ok(())
});

integration_test!(
    test_retry_configuration_integration,
    |host: String, creds: Credentials| -> Result<(), Box<dyn std::error::Error>> {
        // Test retry policy configuration
        let _client = JiraBuilder::new()
            .host(host)
            .credentials(creds)
            .retry_policy(5, Duration::from_millis(500), Duration::from_secs(10))
            .retry_backoff(1.5)
            .build_with_validation()?;

        println!("✓ Retry configuration client created successfully");
        Ok(())
    }
);

integration_test!(test_connection_pool_integration, |host: String,
                                                     creds: Credentials|
 -> Result<
    (),
    Box<dyn std::error::Error>,
> {
    // Test connection pool configuration
    let _client = JiraBuilder::new()
        .host(host)
        .credentials(creds)
        .connection_pool(20, Duration::from_secs(60), true)
        .build_with_validation()?;

    println!("✓ Connection pool configuration client created successfully");
    Ok(())
});

integration_test!(test_caching_integration, |host: String,
                                             creds: Credentials|
 -> Result<
    (),
    Box<dyn std::error::Error>,
> {
    // Test caching configuration
    let _client = JiraBuilder::new()
        .host(host)
        .credentials(creds)
        .memory_cache(Duration::from_secs(300), 1000)
        .build_with_validation()?;

    println!("✓ Caching configuration client created successfully");
    Ok(())
});

integration_test!(test_rate_limiting_integration, |host: String,
                                                   creds: Credentials|
 -> Result<
    (),
    Box<dyn std::error::Error>,
> {
    // Test rate limiting configuration
    let _client = JiraBuilder::new()
        .host(host)
        .credentials(creds)
        .rate_limit(5.0, 10) // Conservative rate limiting for integration tests
        .build_with_validation()?;

    println!("✓ Rate limiting configuration client created successfully");
    Ok(())
});

#[test]
fn test_configuration_validation_errors() {
    // Test that validation catches configuration errors even without real API

    // Invalid timeout
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
        .timeout(Duration::from_secs(0))
        .build_with_validation();
    assert!(result.is_err());
    println!("✓ Invalid timeout correctly rejected");

    // Invalid credentials
    let result = JiraBuilder::new()
        .host("https://test.atlassian.net")
        .credentials(Credentials::Basic("".to_string(), "pass".to_string()))
        .build_with_validation();
    assert!(result.is_err());
    println!("✓ Invalid credentials correctly rejected");

    // Invalid config values
    let mut config = GouqiConfig::default();
    config.retry.max_delay = Duration::from_millis(50);
    config.retry.base_delay = Duration::from_millis(100);
    assert!(config.validate().is_err());
    println!("✓ Invalid retry configuration correctly rejected");
}

#[test]
fn test_configuration_serialization_roundtrip() {
    // Test that configurations can be serialized and deserialized correctly
    use tempfile::NamedTempFile;

    // Test JSON roundtrip
    let original_config = GouqiConfig::high_throughput();
    let temp_file = NamedTempFile::with_suffix(".json").unwrap();

    original_config.save_to_file(temp_file.path()).unwrap();
    let loaded_config = GouqiConfig::from_file(temp_file.path()).unwrap();

    // Compare key values to ensure roundtrip works
    assert_eq!(
        original_config.timeout.default,
        loaded_config.timeout.default
    );
    assert_eq!(
        original_config.connection_pool.max_connections_per_host,
        loaded_config.connection_pool.max_connections_per_host
    );
    assert_eq!(
        original_config.rate_limiting.requests_per_second,
        loaded_config.rate_limiting.requests_per_second
    );

    println!("✓ Configuration serialization roundtrip successful");
}

#[test]
fn test_configuration_merging() {
    // Test configuration merging behavior
    let base_config = GouqiConfig::default();
    let override_config = GouqiConfig::high_throughput();

    let merged = base_config.merge(override_config.clone());

    // Should have high throughput values
    assert_eq!(
        merged.connection_pool.max_connections_per_host,
        override_config.connection_pool.max_connections_per_host
    );
    assert_eq!(
        merged.rate_limiting.requests_per_second,
        override_config.rate_limiting.requests_per_second
    );

    println!("✓ Configuration merging works correctly");
}

// Stress test configuration creation performance
#[test]
fn test_configuration_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Create 100 configurations to test performance
    for i in 0..100 {
        let _config = GouqiConfig::default();
        let _builder = JiraBuilder::new()
            .host(format!("https://test{}.atlassian.net", i))
            .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
            .timeout(Duration::from_secs(30))
            .retry_policy(3, Duration::from_millis(100), Duration::from_secs(10));
        // Note: Not calling build() to avoid actual connection attempts
    }

    let duration = start.elapsed();
    println!(
        "✓ Created 100 configurations in {:?} (avg: {:?})",
        duration,
        duration / 100
    );

    // Should complete in reasonable time (under 100ms)
    assert!(
        duration.as_millis() < 100,
        "Configuration creation too slow: {:?}",
        duration
    );
}

#[test]
fn test_concurrent_configuration_creation() {
    // Test thread safety of configuration creation
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Create 10 threads that each create 10 configurations
    for thread_id in 0..10 {
        let counter_clone = counter.clone();
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let _builder = JiraBuilder::new()
                    .host(format!(
                        "https://thread{}-test{}.atlassian.net",
                        thread_id, i
                    ))
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .timeout(Duration::from_secs(30));

                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 100);
    println!("✓ Concurrent configuration creation successful (100 configs across 10 threads)");
}
