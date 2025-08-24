// Performance benchmarks for configuration system
// Run with: cargo bench --bench configuration_benchmarks

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;
use std::time::Duration;

use gouqi::{ConfigTemplate, Credentials, FieldSchema, GouqiConfig, JiraBuilder};

fn bench_basic_config_creation(c: &mut Criterion) {
    c.bench_function("config_default", |b| {
        b.iter(|| {
            let config = black_box(GouqiConfig::default());
            black_box(config)
        })
    });
}

fn bench_config_templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_templates");

    group.bench_function("high_throughput", |b| {
        b.iter(|| {
            let config = black_box(GouqiConfig::high_throughput());
            black_box(config)
        })
    });

    group.bench_function("low_resource", |b| {
        b.iter(|| {
            let config = black_box(GouqiConfig::low_resource());
            black_box(config)
        })
    });

    group.finish();
}

fn bench_builder_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder_creation");

    group.bench_function("basic", |b| {
        b.iter(|| {
            let builder = black_box(
                JiraBuilder::new()
                    .host("https://test.atlassian.net")
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .timeout(Duration::from_secs(30)),
            );
            black_box(builder)
        })
    });

    group.bench_function("comprehensive", |b| {
        b.iter(|| {
            let builder = black_box(
                JiraBuilder::new()
                    .host("https://test.atlassian.net")
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .timeout(Duration::from_secs(30))
                    .connect_timeout(Duration::from_secs(10))
                    .read_timeout(Duration::from_secs(25))
                    .retry_policy(5, Duration::from_millis(100), Duration::from_secs(30))
                    .retry_backoff(2.0)
                    .connection_pool(20, Duration::from_secs(60), true)
                    .memory_cache(Duration::from_secs(300), 1000)
                    .rate_limit(10.0, 20)
                    .enable_metrics()
                    .user_agent("BenchmarkClient/1.0"),
            );
            black_box(builder)
        })
    });

    group.finish();
}

fn bench_builder_templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder_templates");

    group.bench_function("high_throughput", |b| {
        b.iter(|| {
            let builder = black_box(
                JiraBuilder::new()
                    .host("https://test.atlassian.net")
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .config_template(ConfigTemplate::HighThroughput),
            );
            black_box(builder)
        })
    });

    group.bench_function("low_resource", |b| {
        b.iter(|| {
            let builder = black_box(
                JiraBuilder::new()
                    .host("https://test.atlassian.net")
                    .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                    .config_template(ConfigTemplate::LowResource),
            );
            black_box(builder)
        })
    });

    group.finish();
}

fn bench_environment_loading(c: &mut Criterion) {
    use std::env;

    // Set up environment variables for benchmarking
    unsafe {
        env::set_var("JIRA_TIMEOUT", "45");
        env::set_var("JIRA_MAX_RETRIES", "5");
        env::set_var("JIRA_MAX_CONNECTIONS", "20");
        env::set_var("JIRA_CACHE_ENABLED", "true");
        env::set_var("JIRA_METRICS_ENABLED", "true");
        env::set_var("JIRA_RATE_LIMITING_ENABLED", "true");
    }

    c.bench_function("env_config_load", |b| {
        b.iter(|| {
            let builder = black_box(
                JiraBuilder::new()
                    .config_from_env()
                    .unwrap_or_else(|_| JiraBuilder::new()),
            );
            black_box(builder)
        })
    });

    // Cleanup
    unsafe {
        env::remove_var("JIRA_TIMEOUT");
        env::remove_var("JIRA_MAX_RETRIES");
        env::remove_var("JIRA_MAX_CONNECTIONS");
        env::remove_var("JIRA_CACHE_ENABLED");
        env::remove_var("JIRA_METRICS_ENABLED");
        env::remove_var("JIRA_RATE_LIMITING_ENABLED");
    }
}

fn bench_field_schema_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_schema");

    group.bench_function("simple_text", |b| {
        b.iter(|| {
            let schema = black_box(FieldSchema::text(false));
            black_box(schema)
        })
    });

    group.bench_function("number_with_range", |b| {
        b.iter(|| {
            let schema = black_box(FieldSchema::number(true, Some(0.0), Some(100.0)));
            black_box(schema)
        })
    });

    group.bench_function("enumeration", |b| {
        b.iter(|| {
            let schema = black_box(
                FieldSchema::enumeration(
                    true,
                    vec!["High", "Medium", "Low", "Critical", "Blocker"],
                )
                .unwrap(),
            );
            black_box(schema)
        })
    });

    group.bench_function("complex_custom_field", |b| {
        b.iter(|| {
            let mut custom_fields = HashMap::new();

            // Create multiple field schemas
            custom_fields.insert(
                "story_points".to_string(),
                FieldSchema::number(false, Some(0.0), Some(100.0)),
            );
            custom_fields.insert(
                "priority".to_string(),
                FieldSchema::enumeration(true, vec!["High", "Medium", "Low"]).unwrap(),
            );
            custom_fields.insert(
                "components".to_string(),
                FieldSchema::array(false, "string"),
            );
            custom_fields.insert(
                "is_flagged".to_string(),
                FieldSchema::boolean(false, Some(false)),
            );

            black_box(custom_fields)
        })
    });

    group.finish();
}

fn bench_config_validation(c: &mut Criterion) {
    let valid_config = GouqiConfig::default();
    let mut invalid_config = GouqiConfig::default();
    invalid_config.retry.max_delay = Duration::from_millis(50);
    invalid_config.retry.base_delay = Duration::from_millis(100);

    let mut group = c.benchmark_group("config_validation");

    group.bench_function("valid_config", |b| {
        b.iter(|| {
            let result = black_box(valid_config.validate());
            black_box(result)
        })
    });

    group.bench_function("invalid_config", |b| {
        b.iter(|| {
            let result = black_box(invalid_config.validate());
            black_box(result)
        })
    });

    group.finish();
}

fn bench_config_serialization(c: &mut Criterion) {
    let config = GouqiConfig::high_throughput();

    let mut group = c.benchmark_group("config_serialization");

    group.bench_function("serialize_json", |b| {
        b.iter(|| {
            let json = black_box(serde_json::to_string(&config).unwrap());
            black_box(json)
        })
    });

    let json_str = serde_json::to_string(&config).unwrap();
    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            let config: GouqiConfig = black_box(serde_json::from_str(&json_str).unwrap());
            black_box(config)
        })
    });

    group.finish();
}

fn bench_config_merging(c: &mut Criterion) {
    let base_config = GouqiConfig::default();
    let override_configs = vec![
        GouqiConfig::high_throughput(),
        GouqiConfig::low_resource(),
        GouqiConfig::default(), // Same config
    ];

    let mut group = c.benchmark_group("config_merging");

    for (i, override_config) in override_configs.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("merge", i),
            override_config,
            |b, override_config| {
                b.iter(|| {
                    let merged = black_box(base_config.clone().merge(override_config.clone()));
                    black_box(merged)
                })
            },
        );
    }

    group.finish();
}

fn bench_duration_parsing(c: &mut Criterion) {
    use std::env;

    let duration_strings = vec![
        "30",    // seconds as number
        "45s",   // seconds with suffix
        "500ms", // milliseconds
        "2m",    // minutes
        "1h",    // hours
    ];

    let mut group = c.benchmark_group("duration_parsing");

    for duration_str in &duration_strings {
        // Set environment variable for parsing
        unsafe {
            env::set_var("BENCHMARK_DURATION", duration_str);
        }

        group.bench_with_input(
            BenchmarkId::new("parse", duration_str),
            duration_str,
            |b, _| {
                b.iter(|| {
                    // Test duration parsing through timeout configuration
                    let builder = black_box(
                        JiraBuilder::new()
                            .host("https://test.atlassian.net")
                            .credentials(Credentials::Basic("user".to_string(), "pass".to_string()))
                            .timeout(Duration::from_secs(30)),
                    );
                    black_box(builder)
                })
            },
        );
    }

    // Cleanup
    unsafe {
        env::remove_var("BENCHMARK_DURATION");
    }

    group.finish();
}

// Compare memory usage between different configuration approaches
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Simple configuration
    group.bench_function("simple_config", |b| {
        b.iter(|| {
            let configs: Vec<GouqiConfig> = (0..100)
                .map(|_| black_box(GouqiConfig::default()))
                .collect();
            black_box(configs)
        })
    });

    // Complex configuration with custom fields
    group.bench_function("complex_config", |b| {
        b.iter(|| {
            let configs: Vec<JiraBuilder> = (0..100)
                .map(|i| {
                    let mut custom_fields = HashMap::new();
                    custom_fields.insert(format!("field_{}", i), FieldSchema::text(false));
                    custom_fields.insert(
                        format!("number_{}", i),
                        FieldSchema::number(false, Some(0.0), Some(100.0)),
                    );

                    black_box(
                        JiraBuilder::new()
                            .host(&format!("https://test{}.atlassian.net", i))
                            .credentials(Credentials::Basic(
                                format!("user{}", i),
                                format!("pass{}", i),
                            ))
                            .custom_fields(custom_fields)
                            .config_template(ConfigTemplate::HighThroughput),
                    )
                })
                .collect();
            black_box(configs)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_config_creation,
    bench_config_templates,
    bench_builder_creation,
    bench_builder_templates,
    bench_environment_loading,
    bench_field_schema_creation,
    bench_config_validation,
    bench_config_serialization,
    bench_config_merging,
    bench_duration_parsing,
    bench_memory_usage
);

criterion_main!(benches);
