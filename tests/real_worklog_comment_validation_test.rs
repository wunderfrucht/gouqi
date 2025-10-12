//! Real JIRA Cloud validation test for Worklog.comment() with ADF format
//! Tests the fix for Worklog comment handling with actual JIRA Cloud v3 API

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

#[test]
fn test_real_jira_worklog_comment_v3_adf() {
    // Skip if no token provided
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real worklog validation test");
            return;
        }
    };

    println!("🧪 Testing Worklog.comment() with real JIRA Cloud V3 API...");

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

    // Test 1: Search for issues with worklogs
    println!("🔍 Searching for issues with worklogs...");
    let search_results = jira
        .search()
        .list(
            "worklogDate >= -90d ORDER BY updated DESC",
            &SearchOptions::builder()
                .fields(vec!["worklog", "summary", "key"])
                .max_results(5)
                .build(),
        )
        .expect("Search failed");

    println!("✅ Found {} issues\n", search_results.total);

    let mut tested_worklog_count = 0;
    let mut adf_comment_count = 0;
    let mut string_comment_count = 0;
    let mut no_comment_count = 0;

    for issue in &search_results.issues {
        println!("📝 Testing issue: {}", issue.key);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        // Get worklogs for the issue
        match jira.issues().get_worklogs(&issue.key) {
            Ok(worklogs_response) => {
                println!("   Worklogs: {}", worklogs_response.total);

                for worklog in worklogs_response.worklogs.iter().take(3) {
                    tested_worklog_count += 1;

                    // Test worklog comment extraction
                    match worklog.comment() {
                        Some(comment) => {
                            println!("   ✅ Worklog comment extracted successfully");
                            println!("   Length: {} characters", comment.len());

                            // Show first 80 chars
                            let preview = if comment.len() > 80 {
                                format!("{}...", &comment[..80])
                            } else {
                                comment.clone()
                            };
                            println!("   Preview: {}", preview.replace('\n', " "));

                            // Check if it looks like it came from ADF (has actual text content)
                            if !comment.trim().is_empty() {
                                adf_comment_count += 1;

                                // Validate the comment is properly extracted
                                assert!(
                                    !comment.contains('{') || comment.matches('{').count() < 3,
                                    "Comment should not contain excessive JSON characters"
                                );
                                assert!(
                                    !comment.contains("\"type\":"),
                                    "Comment should not contain ADF structure"
                                );
                                assert!(
                                    !comment.contains("\"content\":"),
                                    "Comment should not contain ADF structure"
                                );
                            } else {
                                string_comment_count += 1;
                            }
                        }
                        None => {
                            no_comment_count += 1;
                            println!("   ℹ️  No comment (returned None)");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Could not fetch worklogs: {}", e);
            }
        }

        println!(); // Blank line between issues
    }

    println!("📊 Validation Summary:");
    println!("   Total worklogs tested: {}", tested_worklog_count);
    println!("   Comments with ADF content: {}", adf_comment_count);
    println!("   Comments with string content: {}", string_comment_count);
    println!("   Worklogs with no comment: {}", no_comment_count);

    // Test 2: Get a specific issue and validate worklogs
    if !search_results.issues.is_empty() {
        let first_key = &search_results.issues[0].key;
        println!("\n🔍 Fetching specific issue with worklogs: {}", first_key);

        let issue = jira.issues().get(first_key).expect("Failed to get issue");
        println!("✅ Issue fetched successfully");

        // Get worklogs directly
        match jira.issues().get_worklogs(&issue.key) {
            Ok(worklogs_response) => {
                println!("✅ Found {} worklogs", worklogs_response.total);

                for (i, worklog) in worklogs_response.worklogs.iter().enumerate().take(3) {
                    println!("\n   Worklog {}/{}:", i + 1, worklogs_response.total);
                    println!("   Time spent: {:?}", worklog.time_spent);
                    println!("   Time spent (seconds): {:?}", worklog.time_spent_seconds);

                    if let Some(author) = &worklog.author {
                        println!("   Author: {}", author.display_name);
                    }

                    if let Some(comment) = worklog.comment() {
                        println!("   Comment length: {} characters", comment.len());

                        // Validate it's plain text, not JSON/ADF structure
                        assert!(
                            !comment.contains('{') || comment.matches('{').count() < 5,
                            "Comment should be plain text, not JSON structure"
                        );

                        println!("   ✅ Comment extracted correctly");
                    } else {
                        println!("   ℹ️  No comment content");
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Could not fetch worklogs: {}", e);
            }
        }
    }

    println!("\n🎉 Worklog comment validation test completed successfully!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_jira_worklog_comment_async() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping async worklog validation");
            return;
        }
    };

    println!("🧪 Testing async Worklog.comment() with real JIRA Cloud...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = gouqi::r#async::Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create async Jira client");

    // Search for issues with worklogs
    let search_results = jira
        .search()
        .list(
            "worklogDate >= -90d ORDER BY updated DESC",
            &SearchOptions::builder()
                .fields(vec!["worklog", "summary"])
                .max_results(3)
                .build(),
        )
        .await
        .expect("Async search failed");

    println!("✅ Async search found {} issues", search_results.total);

    for issue in &search_results.issues {
        println!("\n📝 Testing issue: {}", issue.key);

        if let Ok(worklogs_response) = jira.issues().get_worklogs(&issue.key).await {
            println!("   Worklogs: {}", worklogs_response.total);

            for worklog in worklogs_response.worklogs.iter().take(2) {
                if let Some(comment) = worklog.comment() {
                    println!("   ✅ Worklog comment: {} chars", comment.len());

                    // Validate it's plain text
                    assert!(
                        !comment.contains("\"type\":\"doc\""),
                        "Comment should not contain ADF JSON structure"
                    );
                }
            }
        }
    }

    println!("\n🎉 Async worklog validation completed successfully!");
}
