//! Real JIRA Cloud validation test for Issue::description() with ADF format
//! Tests the fix for issue #122 with actual JIRA Cloud data

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

#[test]
fn test_real_jira_description_v3_adf() {
    // Skip if no token provided
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real description validation test");
            return;
        }
    };

    println!("🧪 Testing Issue::description() with real JIRA Cloud V3 API...");

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

    // Test 1: Search for issues with descriptions
    println!("🔍 Searching for issues with descriptions...");
    let search_results = jira
        .search()
        .list(
            "description is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["description", "summary", "key"])
                .max_results(5)
                .build(),
        )
        .expect("Search failed");

    println!("✅ Found {} issues with descriptions", search_results.total);

    let mut tested_count = 0;
    let mut adf_count = 0;
    let mut string_count = 0;
    let mut empty_count = 0;

    for issue in &search_results.issues {
        tested_count += 1;
        println!("\n📝 Testing issue: {}", issue.key);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        // Test description extraction
        match issue.description() {
            Some(desc) => {
                println!("   ✅ Description extracted successfully");
                println!("   Length: {} characters", desc.len());

                // Show first 100 chars
                let preview = if desc.len() > 100 {
                    format!("{}...", &desc[..100])
                } else {
                    desc.clone()
                };
                println!("   Preview: {}", preview.replace('\n', " "));

                // Check if it looks like it came from ADF (has actual text content)
                if !desc.trim().is_empty() {
                    adf_count += 1;

                    // Validate the description is properly extracted
                    assert!(
                        !desc.contains('{') && !desc.contains('}'),
                        "Description should not contain JSON characters"
                    );
                    assert!(
                        !desc.contains("\"type\":"),
                        "Description should not contain ADF structure"
                    );
                    assert!(
                        !desc.contains("\"content\":"),
                        "Description should not contain ADF structure"
                    );
                } else {
                    string_count += 1;
                }
            }
            None => {
                empty_count += 1;
                println!("   ℹ️  No description (returned None)");
            }
        }
    }

    println!("\n📊 Validation Summary:");
    println!("   Total issues tested: {}", tested_count);
    println!("   Issues with ADF descriptions: {}", adf_count);
    println!("   Issues with string descriptions: {}", string_count);
    println!("   Issues with no description: {}", empty_count);

    // Test 2: Get a specific issue by key and validate description
    if !search_results.issues.is_empty() {
        let first_key = &search_results.issues[0].key;
        println!("\n🔍 Fetching specific issue: {}", first_key);

        let issue = jira.issues().get(first_key).expect("Failed to get issue");

        println!("✅ Issue fetched successfully");

        if let Some(desc) = issue.description() {
            println!("✅ Description extracted from direct fetch");
            println!("   Length: {} characters", desc.len());

            // Validate it's plain text, not JSON/ADF structure
            assert!(
                !desc.contains('{') || desc.matches('{').count() < 5,
                "Description should be plain text, not JSON structure"
            );
        }
    }

    println!("\n🎉 Description validation test completed successfully!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_jira_description_async() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping async description validation");
            return;
        }
    };

    println!("🧪 Testing async Issue::description() with real JIRA Cloud...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = gouqi::r#async::Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create async Jira client");

    // Search for issues with descriptions
    let search_results = jira
        .search()
        .list(
            "description is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["description", "summary"])
                .max_results(3)
                .build(),
        )
        .await
        .expect("Async search failed");

    println!("✅ Async search found {} issues", search_results.total);

    for issue in &search_results.issues {
        println!("\n📝 Testing issue: {}", issue.key);

        if let Some(desc) = issue.description() {
            println!("   ✅ Description: {} chars", desc.len());

            // Validate it's plain text
            assert!(
                !desc.contains("\"type\":\"doc\""),
                "Description should not contain ADF JSON structure"
            );
        }
    }

    println!("\n🎉 Async description validation completed successfully!");
}

#[test]
fn test_description_multiline_handling() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping multiline test");
            return;
        }
    };

    println!("🧪 Testing multiline description handling...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Search for issues and check for multiline descriptions
    let search_results = jira
        .search()
        .list(
            "description is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["description"])
                .max_results(10)
                .build(),
        )
        .expect("Search failed");

    let mut multiline_count = 0;

    for issue in &search_results.issues {
        if let Some(desc) = issue.description() {
            if desc.contains('\n') {
                multiline_count += 1;
                println!("✅ Issue {} has multiline description", issue.key);

                let lines: Vec<&str> = desc.lines().collect();
                println!("   Number of lines: {}", lines.len());

                // Validate that newlines are preserved from ADF paragraphs
                assert!(
                    lines.len() > 1,
                    "Multiline description should have multiple lines"
                );
            }
        }
    }

    println!(
        "\n📊 Found {} issues with multiline descriptions",
        multiline_count
    );
    println!("🎉 Multiline handling test completed!");
}

#[test]
fn test_description_with_formatting() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping formatting test");
            return;
        }
    };

    println!("🧪 Testing description with formatting (bold, italic, etc.)...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    let search_results = jira
        .search()
        .list(
            "description is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["description"])
                .max_results(10)
                .build(),
        )
        .expect("Search failed");

    println!(
        "✅ Testing {} issues for formatting extraction",
        search_results.issues.len()
    );

    for issue in &search_results.issues {
        if let Some(desc) = issue.description() {
            // The description should be plain text - no markdown or HTML tags
            // from ADF formatting marks
            println!("📝 Issue {}: {} chars", issue.key, desc.len());

            // Validate no ADF markup leaked through
            assert!(
                !desc.contains("\"marks\":"),
                "Description should not contain ADF marks structure"
            );
            assert!(
                !desc.contains("\"type\":\"strong\""),
                "Description should not contain ADF strong mark"
            );
            assert!(
                !desc.contains("\"type\":\"em\""),
                "Description should not contain ADF em mark"
            );

            // It's plain text, so it should be readable
            if !desc.trim().is_empty() {
                println!("   ✅ Plain text extracted successfully");
            }
        }
    }

    println!("🎉 Formatting extraction test completed!");
}
