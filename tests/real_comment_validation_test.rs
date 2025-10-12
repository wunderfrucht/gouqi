//! Real JIRA Cloud validation test for Comment.body() with ADF format
//! Tests the fix for Comment body handling with actual JIRA Cloud v3 API

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

#[test]
fn test_real_jira_comment_body_v3_adf() {
    // Skip if no token provided
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real comment validation test");
            return;
        }
    };

    println!("🧪 Testing Comment.body() with real JIRA Cloud V3 API...");

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

    // Test 1: Search for issues with comments
    println!("🔍 Searching for issues with comments...");
    let search_results = jira
        .search()
        .list(
            "comment is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["comment", "summary", "key"])
                .max_results(5)
                .build(),
        )
        .expect("Search failed");

    println!("✅ Found {} issues with comments", search_results.total);

    let mut tested_count = 0;
    let mut adf_count = 0;
    let mut empty_count = 0;

    for issue in &search_results.issues {
        tested_count += 1;
        println!("\n📝 Testing issue: {}", issue.key);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        // Get comments from the issue
        if let Some(comments) = issue.comments() {
            println!("   Comments: {}", comments.total);

            for comment in &comments.comments {
                // Test comment body extraction
                let body = &*comment.body;

                println!("   ✅ Comment body extracted successfully");
                println!("   Length: {} characters", body.len());

                // Show first 100 chars
                let preview = if body.len() > 100 {
                    format!("{}...", &body[..100])
                } else {
                    body.to_string()
                };
                println!("   Preview: {}", preview.replace('\n', " "));

                // Check if it looks like it came from ADF (has actual text content)
                if !body.trim().is_empty() {
                    adf_count += 1;

                    // Validate the body is properly extracted
                    assert!(
                        !body.contains('{') || body.matches('{').count() < 3,
                        "Body should not contain excessive JSON characters"
                    );
                    assert!(
                        !body.contains("\"type\":"),
                        "Body should not contain ADF structure"
                    );
                    assert!(
                        !body.contains("\"content\":"),
                        "Body should not contain ADF structure"
                    );
                } else {
                    empty_count += 1;
                }
            }
        } else {
            println!("   ℹ️  No comments found");
        }
    }

    println!("\n📊 Validation Summary:");
    println!("   Total issues tested: {}", tested_count);
    println!("   Comments with ADF bodies: {}", adf_count);
    println!("   Comments with empty body: {}", empty_count);

    // Test 2: Get a specific issue and validate comments
    if !search_results.issues.is_empty() {
        let first_key = &search_results.issues[0].key;
        println!("\n🔍 Fetching specific issue with comments: {}", first_key);

        let issue = jira.issues().get(first_key).expect("Failed to get issue");

        println!("✅ Issue fetched successfully");

        if let Some(comments) = issue.comments() {
            println!("✅ Found {} comments", comments.total);

            for (i, comment) in comments.comments.iter().enumerate().take(3) {
                println!("\n   Comment {}/{}:", i + 1, comments.total);

                if let Some(author) = &comment.author {
                    println!("   Author: {}", author.display_name);
                }

                let body = &*comment.body;
                println!("   Body length: {} characters", body.len());

                // Validate it's plain text, not JSON/ADF structure
                assert!(
                    !body.contains('{') || body.matches('{').count() < 5,
                    "Body should be plain text, not JSON structure"
                );

                println!("   ✅ Body extracted correctly");
            }
        }
    }

    println!("\n🎉 Comment body validation test completed successfully!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_jira_comment_body_async() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping async comment validation");
            return;
        }
    };

    println!("🧪 Testing async Comment.body() with real JIRA Cloud...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = gouqi::r#async::Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create async Jira client");

    // Search for issues with comments
    let search_results = jira
        .search()
        .list(
            "comment is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["comment", "summary"])
                .max_results(3)
                .build(),
        )
        .await
        .expect("Async search failed");

    println!("✅ Async search found {} issues", search_results.total);

    for issue in &search_results.issues {
        println!("\n📝 Testing issue: {}", issue.key);

        if let Some(comments) = issue.comments() {
            for comment in &comments.comments {
                let body = &*comment.body;
                println!("   ✅ Comment body: {} chars", body.len());

                // Validate it's plain text
                assert!(
                    !body.contains("\"type\":\"doc\""),
                    "Body should not contain ADF JSON structure"
                );
            }
        }
    }

    println!("\n🎉 Async comment validation completed successfully!");
}

#[test]
fn test_comment_body_multiline_handling() {
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping multiline test");
            return;
        }
    };

    println!("🧪 Testing multiline comment body handling...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Search for issues and check for multiline comments
    let search_results = jira
        .search()
        .list(
            "comment is not EMPTY ORDER BY created DESC",
            &SearchOptions::builder()
                .fields(vec!["comment"])
                .max_results(10)
                .build(),
        )
        .expect("Search failed");

    let mut multiline_count = 0;

    for issue in &search_results.issues {
        if let Some(comments) = issue.comments() {
            for comment in &comments.comments {
                let body = &*comment.body;
                if body.contains('\n') {
                    multiline_count += 1;
                    println!("✅ Issue {} comment has multiline body", issue.key);

                    let lines: Vec<&str> = body.lines().collect();
                    println!("   Number of lines: {}", lines.len());

                    // Validate that newlines are preserved from ADF paragraphs
                    assert!(lines.len() > 1, "Multiline body should have multiple lines");
                }
            }
        }
    }

    println!(
        "\n📊 Found {} comments with multiline bodies",
        multiline_count
    );
    println!("🎉 Multiline handling test completed!");
}
