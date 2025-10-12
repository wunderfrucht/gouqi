//! Real JIRA Cloud validation test for Issue.environment() with ADF format
//! Tests the fix for environment field handling with actual JIRA Cloud v3 API

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

#[test]
fn test_real_jira_environment_v3_adf() {
    // Skip if no token provided
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real environment validation test");
            return;
        }
    };

    println!("🧪 Testing Issue.environment() with real JIRA Cloud V3 API...");

    // Get username from environment
    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());

    // Create client for gouji.atlassian.net (V3 API) with Basic auth
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Verify V3 API is being used
    assert_eq!(jira.get_search_api_version(), gouqi::SearchApiVersion::V3);
    println!("✅ Confirmed V3 API auto-detection");

    // Test 1: Search for issues with environment field populated
    println!("🔍 Searching for issues with environment field...");
    let search_results = jira
        .search()
        .list(
            "environment is not EMPTY ORDER BY updated DESC",
            &SearchOptions::builder()
                .fields(vec!["environment", "summary", "key"])
                .max_results(5)
                .build(),
        )
        .expect("Search failed");

    println!(
        "✅ Found {} issues with environment field\n",
        search_results.total
    );

    let mut tested_count = 0;
    let mut adf_count = 0;
    let mut string_count = 0;
    let mut empty_count = 0;

    for issue in &search_results.issues {
        tested_count += 1;
        println!("📝 Testing issue: {}", issue.key);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        // Test environment extraction
        match issue.environment() {
            Some(environment) => {
                println!("   ✅ Environment extracted successfully");
                println!("   Length: {} characters", environment.len());

                // Show first 100 chars
                let preview = if environment.len() > 100 {
                    format!("{}...", &environment[..100])
                } else {
                    environment.clone()
                };
                println!("   Preview: {}", preview.replace('\n', " "));

                // Check if it looks like it came from ADF (has actual text content)
                if !environment.trim().is_empty() {
                    adf_count += 1;

                    // Validate the environment is properly extracted
                    assert!(
                        !environment.contains('{') || environment.matches('{').count() < 3,
                        "Environment should not contain excessive JSON characters"
                    );
                    assert!(
                        !environment.contains("\"type\":"),
                        "Environment should not contain ADF structure"
                    );
                    assert!(
                        !environment.contains("\"content\":"),
                        "Environment should not contain ADF structure"
                    );
                } else {
                    string_count += 1;
                }
            }
            None => {
                empty_count += 1;
                println!("   ℹ️  No environment (returned None)");
            }
        }

        println!(); // Blank line between issues
    }

    println!("📊 Validation Summary:");
    println!("   Total issues tested: {}", tested_count);
    println!("   Issues with ADF environment: {}", adf_count);
    println!("   Issues with string environment: {}", string_count);
    println!("   Issues with no environment: {}", empty_count);

    // Test 2: Get a specific issue and validate environment
    if !search_results.issues.is_empty() {
        let first_key = &search_results.issues[0].key;
        println!(
            "\n🔍 Fetching specific issue with environment: {}",
            first_key
        );

        let issue = jira.issues().get(first_key).expect("Failed to get issue");
        println!("✅ Issue fetched successfully");

        if let Some(environment) = issue.environment() {
            println!("✅ Environment: {} characters", environment.len());

            // Validate it's plain text, not JSON/ADF structure
            assert!(
                !environment.contains('{') || environment.matches('{').count() < 5,
                "Environment should be plain text, not JSON structure"
            );

            println!("✅ Environment extracted correctly");
        } else {
            println!("ℹ️  No environment content");
        }
    }

    println!("\n🎉 Environment validation test completed successfully!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_jira_environment_async() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping async environment validation");
            return;
        }
    };

    println!("🧪 Testing async Issue.environment() with real JIRA Cloud...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = gouqi::r#async::Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create async Jira client");

    // Search for issues with environment
    let search_results = jira
        .search()
        .list(
            "environment is not EMPTY ORDER BY updated DESC",
            &SearchOptions::builder()
                .fields(vec!["environment", "summary"])
                .max_results(3)
                .build(),
        )
        .await
        .expect("Async search failed");

    println!("✅ Async search found {} issues", search_results.total);

    for issue in &search_results.issues {
        println!("\n📝 Testing issue: {}", issue.key);

        if let Some(environment) = issue.environment() {
            println!("   ✅ Environment: {} chars", environment.len());

            // Validate it's plain text
            assert!(
                !environment.contains("\"type\":\"doc\""),
                "Environment should not contain ADF JSON structure"
            );
        }
    }

    println!("\n🎉 Async environment validation completed successfully!");
}
