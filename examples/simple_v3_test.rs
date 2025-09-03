//! Simple V3 test to verify basic functionality
//!
//! Set INTEGRATION_JIRA_TOKEN and optionally JIRA_USER
//! Run with: cargo run --example simple_v3_test

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("INTEGRATION_JIRA_TOKEN")
        .expect("Please set INTEGRATION_JIRA_TOKEN environment variable");

    // Try different authentication methods
    println!("🔐 Testing different authentication methods...");

    // Method 1: Bearer token (what we were trying)
    println!("\n1️⃣  Bearer Token Authentication");
    test_auth(
        "https://gouji.atlassian.net",
        Credentials::Bearer(token.clone()),
    )?;

    // Method 2: Basic auth with email + token (more common for Jira Cloud)
    if let Ok(user) = env::var("JIRA_USER") {
        println!("\n2️⃣  Basic Authentication (email + token)");
        test_auth(
            "https://gouji.atlassian.net",
            Credentials::Basic(user, token.clone()),
        )?;
    } else {
        println!("\n2️⃣  Skipping Basic auth (no JIRA_USER set)");
    }

    Ok(())
}

fn test_auth(host: &str, credentials: Credentials) -> Result<(), Box<dyn std::error::Error>> {
    match credentials {
        Credentials::Bearer(_) => println!("   Using Bearer token..."),
        Credentials::Basic(ref user, _) => println!("   Using Basic auth for user: {}", user),
        _ => println!("   Using other credentials..."),
    }

    let jira = Jira::new(host, credentials)?;

    println!("   🔧 API Version: {:?}", jira.get_search_api_version());

    // Test simple search with our V3 auto-injection
    println!("   🔍 Testing V3 search with auto field injection...");

    let result = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder().max_results(1).build(),
    );

    match result {
        Ok(search_results) => {
            println!("   ✅ Search successful!");
            println!("   📊 Total issues: {}", search_results.total);

            if !search_results.issues.is_empty() {
                let issue = &search_results.issues[0];
                println!(
                    "   📋 First issue: {} - {}",
                    issue.key,
                    issue.summary().unwrap_or("No summary".to_string())
                );

                // Test our auto-injection worked
                println!("   🔧 Fields populated:");
                println!("      - ID: {} (len: {})", issue.id, issue.id.len());
                println!("      - Key: {} (len: {})", issue.key, issue.key.len());
                println!(
                    "      - Self: {} (len: {})",
                    issue.self_link,
                    issue.self_link.len()
                );
                println!("      - Fields count: {}", issue.fields.len());
            } else {
                println!("   📝 No issues found (empty instance)");
            }
        }
        Err(e) => {
            println!("   ❌ Search failed: {:?}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}
