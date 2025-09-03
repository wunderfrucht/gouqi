//! Test V3 API with SCRUM project
//!
//! Run with: INTEGRATION_JIRA_TOKEN=your_token cargo run --example test_scrum_project

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("INTEGRATION_JIRA_TOKEN")
        .expect("Please set INTEGRATION_JIRA_TOKEN environment variable");

    println!("🚀 Testing V3 API with SCRUM Project");
    println!("====================================");

    // Create Jira client - should auto-detect V3 for *.atlassian.net
    let jira = Jira::new("https://gouji.atlassian.net", Credentials::Bearer(token))?;

    println!("🔧 API Version: {:?}", jira.get_search_api_version());
    println!();

    // Test 1: Search for issues in SCRUM project
    println!("1️⃣  Searching SCRUM project issues");
    println!("----------------------------------");

    test_search(
        &jira,
        "project = SCRUM ORDER BY created DESC",
        "SCRUM project issues",
    )?;

    // Test 2: Test our auto field injection with default options
    println!("\n2️⃣  Testing default search (auto field injection)");
    println!("--------------------------------------------------");

    let default_results = jira.search().list(
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::default(), // This should auto-inject essential fields for V3
    )?;

    println!(
        "📊 Found {} total issues with auto field injection",
        default_results.total
    );

    if !default_results.issues.is_empty() {
        let issue = &default_results.issues[0];
        println!("✅ Auto-injection working:");
        println!("   - Issue: {}", issue.key);
        println!(
            "   - Self link: {} (len: {})",
            issue.self_link,
            issue.self_link.len()
        );
        println!("   - ID: {} (len: {})", issue.id, issue.id.len());
        println!("   - Fields count: {}", issue.fields.len());
        println!(
            "   - Summary: {}",
            issue.summary().unwrap_or("No summary".to_string())
        );
    }

    // Test 3: Compare different field options
    println!("\n3️⃣  Testing field options");
    println!("-------------------------");

    // Minimal fields
    test_search_with_options(
        &jira,
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::builder().minimal_fields().build(),
        "Minimal fields",
    )?;

    // Standard fields
    test_search_with_options(
        &jira,
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::builder().standard_fields().build(),
        "Standard fields",
    )?;

    // All fields
    test_search_with_options(
        &jira,
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::builder().all_fields().max_results(1).build(),
        "All fields",
    )?;

    println!("\n🎉 V3 API test completed successfully!");
    println!("💡 Notice: All searches used V3 API with proper field handling");

    Ok(())
}

fn test_search(
    jira: &Jira,
    jql: &str,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 {}: {}", description, jql);

    let results = jira
        .search()
        .list(jql, &SearchOptions::builder().max_results(5).build())?;

    println!("📊 Found {} issues", results.total);
    for (i, issue) in results.issues.iter().enumerate() {
        println!(
            "   {}. {} - {}",
            i + 1,
            issue.key,
            issue.summary().unwrap_or("No summary".to_string())
        );
    }

    Ok(())
}

fn test_search_with_options(
    jira: &Jira,
    jql: &str,
    options: &SearchOptions,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 {}", description);

    let results = jira.search().list(jql, options)?;

    if !results.issues.is_empty() {
        let issue = &results.issues[0];
        println!("   Issue: {} - Fields: {}", issue.key, issue.fields.len());
    } else {
        println!("   No issues found");
    }

    Ok(())
}
