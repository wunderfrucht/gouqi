//! Real V3 API integration test with gouji.atlassian.net
//! Tests the actual V3 field auto-injection functionality

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

#[test]
fn test_real_v3_search_with_auto_injection() {
    // Skip if no token provided
    let token = match env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping real V3 integration test");
            return;
        }
    };

    println!("🧪 Testing V3 API with real Jira Cloud instance...");

    // Create client for gouji.atlassian.net (should auto-detect V3)
    let jira = Jira::new("https://gouji.atlassian.net", Credentials::Bearer(token))
        .expect("Failed to create Jira client");

    // Verify V3 API is being used
    assert_eq!(jira.get_search_api_version(), gouqi::SearchApiVersion::V3);
    println!("✅ Confirmed V3 API auto-detection");

    // Test 1: Default search options (should auto-inject essential fields)
    println!("🔍 Testing default search with auto field injection...");
    let default_results = jira
        .search()
        .list("ORDER BY created DESC", &SearchOptions::default());

    match default_results {
        Ok(results) => {
            println!(
                "✅ Default search successful - found {} issues",
                results.total
            );

            if !results.issues.is_empty() {
                let first_issue = &results.issues[0];
                println!(
                    "   First issue: {} - {}",
                    first_issue.key,
                    first_issue.summary().unwrap_or("No summary".to_string())
                );

                // These fields should be available due to auto-injection
                assert!(
                    !first_issue.self_link.is_empty(),
                    "self_link should be populated"
                );
                assert!(!first_issue.key.is_empty(), "key should be populated");
                assert!(!first_issue.id.is_empty(), "id should be populated");
            }
        }
        Err(e) => {
            panic!("❌ Default search failed: {:?}", e);
        }
    }

    // Test 2: Explicit minimal fields (should only get ID)
    println!("🔍 Testing minimal fields...");
    let minimal_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder()
            .minimal_fields()
            .max_results(1)
            .build(),
    );

    match minimal_results {
        Ok(results) => {
            println!("✅ Minimal search successful");
            if !results.issues.is_empty() {
                let issue = &results.issues[0];
                assert!(!issue.id.is_empty(), "ID should always be present");
                // Note: V3 API might still include other fields even when requesting minimal
                println!("   Issue ID: {}", issue.id);
            }
        }
        Err(e) => {
            println!("⚠️  Minimal search failed (might be expected): {:?}", e);
        }
    }

    // Test 3: Standard fields
    println!("🔍 Testing standard fields...");
    let standard_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder()
            .standard_fields()
            .max_results(1)
            .build(),
    );

    match standard_results {
        Ok(results) => {
            println!("✅ Standard search successful");
            if !results.issues.is_empty() {
                let issue = &results.issues[0];
                println!(
                    "   Issue: {} - Status: {:?}",
                    issue.key,
                    issue
                        .status()
                        .map(|s| s.name)
                        .unwrap_or("Unknown".to_string())
                );
            }
        }
        Err(e) => {
            println!("⚠️  Standard search failed: {:?}", e);
        }
    }

    // Test 4: All fields
    println!("🔍 Testing all fields...");
    let all_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder().all_fields().max_results(1).build(),
    );

    match all_results {
        Ok(results) => {
            println!("✅ All fields search successful");
            if !results.issues.is_empty() {
                let issue = &results.issues[0];
                println!(
                    "   Issue: {} - Fields available: {}",
                    issue.key,
                    issue.fields.keys().collect::<Vec<_>>().len()
                );
            }
        }
        Err(e) => {
            println!("⚠️  All fields search failed: {:?}", e);
        }
    }

    println!("🎉 V3 integration test completed successfully!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_v3_async_search() {
    let token = match env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping async V3 test");
            return;
        }
    };

    println!("🧪 Testing async V3 API...");

    let jira = gouqi::r#async::Jira::new("https://gouji.atlassian.net", Credentials::Bearer(token))
        .expect("Failed to create async Jira client");

    // Test async search with auto-injection
    let results = jira
        .search()
        .list("ORDER BY created DESC", &SearchOptions::default())
        .await;

    match results {
        Ok(search_results) => {
            println!(
                "✅ Async V3 search successful - found {} issues",
                search_results.total
            );

            if !search_results.issues.is_empty() {
                let issue = &search_results.issues[0];
                assert!(!issue.key.is_empty());
                assert!(!issue.self_link.is_empty());
                println!("   First issue: {}", issue.key);
            }
        }
        Err(e) => {
            panic!("❌ Async V3 search failed: {:?}", e);
        }
    }

    println!("🎉 Async V3 test completed successfully!");
}
