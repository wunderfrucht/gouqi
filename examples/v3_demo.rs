//! Demo of V3 API functionality with real Jira Cloud
//!
//! Run with: INTEGRATION_JIRA_TOKEN=your_token cargo run --example v3_demo

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging (optional)
    // tracing_subscriber::fmt::init();

    let token = env::var("INTEGRATION_JIRA_TOKEN")
        .expect("Please set INTEGRATION_JIRA_TOKEN environment variable");

    if token.trim().is_empty() {
        panic!("INTEGRATION_JIRA_TOKEN cannot be empty");
    }

    println!("🚀 gouqi V3 API Demo");
    println!("===================");

    // Create Jira client - should auto-detect V3 for *.atlassian.net
    let jira = Jira::new("https://gouji.atlassian.net", Credentials::Bearer(token))?;

    println!("📡 Connected to: https://gouji.atlassian.net");
    println!("🔧 API Version: {:?}", jira.get_search_api_version());

    // Demo 1: Default behavior (auto field injection for V3)
    println!("\n1️⃣  Default Search (Auto Field Injection)");
    println!("==========================================");

    let default_results = jira
        .search()
        .list("ORDER BY created DESC", &SearchOptions::default())?;

    println!("📊 Found {} total issues", default_results.total);
    println!(
        "📋 Showing first {} issues:",
        default_results.issues.len().min(3)
    );

    for (i, issue) in default_results.issues.iter().take(3).enumerate() {
        println!(
            "   {}. {} - {}",
            i + 1,
            issue.key,
            issue.summary().unwrap_or("No summary".to_string())
        );

        if let Some(status) = issue.status() {
            println!("      Status: {}", status.name);
        }

        if let Some(assignee) = issue.assignee() {
            println!("      Assignee: {}", assignee.display_name);
        }
    }

    // Demo 2: Minimal fields for performance
    println!("\n2️⃣  Minimal Fields (Performance Optimized)");
    println!("===========================================");

    let minimal_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder()
            .minimal_fields()
            .max_results(5)
            .build(),
    )?;

    println!(
        "📊 Found {} issues with minimal data",
        minimal_results.issues.len()
    );
    for issue in &minimal_results.issues {
        println!("   • ID: {} Key: {}", issue.id, issue.key);
    }

    // Demo 3: Standard fields for common use cases
    println!("\n3️⃣  Standard Fields (Common Use Case)");
    println!("======================================");

    let standard_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder()
            .standard_fields()
            .max_results(3)
            .build(),
    )?;

    for issue in &standard_results.issues {
        println!("📋 Issue: {}", issue.key);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        if let Some(status) = issue.status() {
            println!("   Status: {}", status.name);
        }

        if let Some(assignee) = issue.assignee() {
            println!("   Assignee: {}", assignee.display_name);
        }

        if let Some(created) = issue.created() {
            println!(
                "   Created: {}",
                created.format(&time::format_description::well_known::Iso8601::DEFAULT)?
            );
        }
        println!();
    }

    // Demo 4: Custom field selection
    println!("4️⃣  Custom Field Selection");
    println!("==========================");

    let custom_results = jira.search().list(
        "ORDER BY created DESC",
        &SearchOptions::builder()
            .fields(vec!["id", "key", "summary", "status", "priority"])
            .max_results(2)
            .build(),
    )?;

    for issue in &custom_results.issues {
        println!(
            "🎯 Issue: {} - {}",
            issue.key,
            issue.summary().unwrap_or("N/A".to_string())
        );

        if let Some(priority) = issue.priority() {
            println!("   Priority: {}", priority.name);
        }
    }

    println!("\n✅ V3 Demo completed successfully!");
    println!("💡 Notice: All searches worked seamlessly with V3 API auto-detection!");

    Ok(())
}
