// Thread safety verification tests
// These tests ensure that configuration creation, environment variable access,
// and other operations are safe under concurrent access

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use gouqi::{Credentials, FieldSchema, GouqiConfig, JiraBuilder};

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
fn test_concurrent_builder_creation() {
    // Test that multiple threads can create builders simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for thread_id in 0..10 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Create multiple builders in each thread
            for i in 0..50 {
                let _builder = JiraBuilder::new()
                    .host(format!(
                        "https://thread{}-test{}.atlassian.net",
                        thread_id, i
                    ))
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .timeout(Duration::from_secs(30))
                    .retry_policy(3, Duration::from_millis(100), Duration::from_secs(10));

                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 500);
}

#[test]
fn test_concurrent_config_creation() {
    // Test that multiple threads can create configurations simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(8));
    let mut handles = vec![];

    for template_type in 0..8 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for _ in 0..100 {
                let _config = match template_type % 3 {
                    0 => GouqiConfig::default(),
                    1 => GouqiConfig::high_throughput(),
                    _ => GouqiConfig::low_resource(),
                };
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 800);
}

#[test]
fn test_concurrent_field_schema_creation() {
    // Test that multiple threads can create field schemas simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(6));
    let mut handles = vec![];

    for schema_type in 0..6 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for i in 0..100 {
                let _schema = match schema_type % 6 {
                    0 => FieldSchema::text(false),
                    1 => FieldSchema::number(false, Some(0.0), Some(100.0)),
                    2 => FieldSchema::integer(false, Some(0), Some(100)),
                    3 => FieldSchema::boolean(false, Some(true)),
                    4 => FieldSchema::date(false),
                    _ => FieldSchema::enumeration(
                        false,
                        vec![
                            format!("value_{}_{}", schema_type, i),
                            "option1".to_string(),
                            "option2".to_string(),
                        ],
                    )
                    .expect("Failed to create enumeration"),
                };
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 600);
}

#[test]
fn test_concurrent_environment_variable_access() {
    // Test concurrent access to environment variables
    // This test isolates environment variable operations to avoid interference

    let barrier = Arc::new(Barrier::new(5));
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for thread_id in 0..5 {
        let barrier_clone = barrier.clone();
        let success_count_clone = success_count.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Each thread uses its own set of environment variables to avoid conflicts
            let env_vars = [
                (
                    format!("THREAD_{}_JIRA_HOST", thread_id),
                    "https://test.atlassian.net".to_string(),
                ),
                (
                    format!("THREAD_{}_JIRA_USER", thread_id),
                    "testuser".to_string(),
                ),
                (
                    format!("THREAD_{}_JIRA_PASS", thread_id),
                    "testpass".to_string(),
                ),
                (
                    format!("THREAD_{}_JIRA_TIMEOUT", thread_id),
                    "45".to_string(),
                ),
                (
                    format!("THREAD_{}_JIRA_MAX_RETRIES", thread_id),
                    "5".to_string(),
                ),
            ];

            // Set environment variables
            for (key, value) in &env_vars {
                set_test_env_var(key, value);
            }

            // Create configuration using environment (though it won't use thread-specific vars)
            for _ in 0..10 {
                let _config = GouqiConfig::default(); // Uses global env vars, but that's OK
                success_count_clone.fetch_add(1, Ordering::SeqCst);
            }

            // Clean up thread-specific environment variables
            for (key, _) in &env_vars {
                remove_test_env_var(key);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(success_count.load(Ordering::SeqCst), 50);
}

#[test]
fn test_concurrent_config_serialization() {
    // Test that multiple threads can serialize/deserialize configurations simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            let config = match thread_id % 3 {
                0 => GouqiConfig::default(),
                1 => GouqiConfig::high_throughput(),
                _ => GouqiConfig::low_resource(),
            };

            for _ in 0..50 {
                // Serialize to JSON
                let json = serde_json::to_string(&config).expect("Failed to serialize config");

                // Deserialize from JSON
                let _deserialized: GouqiConfig =
                    serde_json::from_str(&json).expect("Failed to deserialize config");

                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 200);
}

#[test]
fn test_concurrent_config_validation() {
    // Test that multiple threads can validate configurations simultaneously
    let valid_counter = Arc::new(AtomicUsize::new(0));
    let invalid_counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let valid_counter_clone = valid_counter.clone();
        let invalid_counter_clone = invalid_counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for i in 0..100 {
                let mut config = GouqiConfig::default();

                // Make some configurations invalid
                if (thread_id + i) % 3 == 0 {
                    // Invalid: base delay > max delay
                    config.retry.base_delay = Duration::from_millis(1000);
                    config.retry.max_delay = Duration::from_millis(500);

                    if config.validate().is_err() {
                        invalid_counter_clone.fetch_add(1, Ordering::SeqCst);
                    }
                } else {
                    // Valid configuration
                    if config.validate().is_ok() {
                        valid_counter_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // We should have roughly 1/3 invalid and 2/3 valid configurations
    assert!(valid_counter.load(Ordering::SeqCst) > 200);
    assert!(invalid_counter.load(Ordering::SeqCst) > 100);
}

#[test]
fn test_concurrent_custom_field_creation() {
    // Test that multiple threads can create custom fields simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(6));
    let mut handles = vec![];

    for thread_id in 0..6 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for i in 0..50 {
                let mut custom_fields = HashMap::new();

                // Create several custom fields per iteration
                custom_fields.insert(
                    format!("text_field_{}_{}", thread_id, i),
                    FieldSchema::text(false),
                );
                custom_fields.insert(
                    format!("number_field_{}_{}", thread_id, i),
                    FieldSchema::number(false, Some(0.0), Some(100.0)),
                );
                custom_fields.insert(
                    format!("bool_field_{}_{}", thread_id, i),
                    FieldSchema::boolean(false, Some(false)),
                );

                let _builder = JiraBuilder::new()
                    .host(format!("https://thread{}.atlassian.net", thread_id))
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .custom_fields(custom_fields);

                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 300);
}

#[test]
fn test_concurrent_config_merging() {
    // Test that multiple threads can merge configurations simultaneously
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    let base_config = Arc::new(GouqiConfig::default());
    let high_throughput_config = Arc::new(GouqiConfig::high_throughput());
    let low_resource_config = Arc::new(GouqiConfig::low_resource());

    for thread_id in 0..4 {
        let counter_clone = counter.clone();
        let barrier_clone = barrier.clone();
        let base_clone = base_config.clone();
        let high_clone = high_throughput_config.clone();
        let low_clone = low_resource_config.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for i in 0..100 {
                let override_config = match (thread_id + i) % 3 {
                    0 => &*base_clone,
                    1 => &*high_clone,
                    _ => &*low_clone,
                };

                let _merged = (*base_clone).clone().merge((*override_config).clone());
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 400);
}

#[test]
fn test_stress_concurrent_operations() {
    // Stress test with multiple concurrent operations
    let operation_count = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(8));
    let mut handles = vec![];

    for thread_id in 0..8 {
        let operation_count_clone = operation_count.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for i in 0..50 {
                // Mix different operations
                match (thread_id + i) % 5 {
                    0 => {
                        // Config creation
                        let _config = GouqiConfig::default();
                    }
                    1 => {
                        // Builder creation
                        let _builder = JiraBuilder::new()
                            .host("https://test.atlassian.net")
                            .credentials(Credentials::Basic(
                                "user".to_string(),
                                "pass".to_string(),
                            ));
                    }
                    2 => {
                        // Field schema creation
                        let _schema = FieldSchema::enumeration(
                            false,
                            vec!["A".to_string(), "B".to_string(), "C".to_string()],
                        )
                        .expect("Failed to create enum");
                    }
                    3 => {
                        // Config validation
                        let config = GouqiConfig::high_throughput();
                        let _ = config.validate();
                    }
                    _ => {
                        // Config serialization
                        let config = GouqiConfig::low_resource();
                        let json = serde_json::to_string(&config).expect("Failed to serialize");
                        let _: GouqiConfig =
                            serde_json::from_str(&json).expect("Failed to deserialize");
                    }
                }

                operation_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(operation_count.load(Ordering::SeqCst), 400);
}
