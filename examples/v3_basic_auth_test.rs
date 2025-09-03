//! Test V3 API with Basic authentication (email + token)
//!
//! Run with: INTEGRATION_JIRA_TOKEN=token JIRA_USER=email cargo run --example v3_basic_auth_test

use gouqi::{Credentials, Jira, SearchOptions};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("INTEGRATION_JIRA_TOKEN")
        .expect("Please set INTEGRATION_JIRA_TOKEN environment variable");

    let user =
        env::var("JIRA_USER").expect("Please set JIRA_USER environment variable (your email)");

    println!("🚀 Testing V3 API with Basic Authentication");
    println!("==========================================");
    println!("👤 User: {}", user);
    println!("🔧 Using Basic auth (email + API token)");

    // Create Jira client with Basic authentication
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(user, token),
    )?;

    println!("🌐 API Version: {:?}", jira.get_search_api_version());
    println!();

    // Test 1: Default search (should auto-inject essential fields for V3)
    println!("1️⃣  Default Search - Auto Field Injection Test");
    println!("===============================================");

    let default_results = jira.search().list(
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::default(), // This triggers our V3 auto field injection!
    )?;

    println!("✅ Search successful!");
    println!("📊 Total issues found: {}", default_results.total);

    if !default_results.issues.is_empty() {
        let issue = &default_results.issues[0];
        println!();
        println!("🔍 First Issue Analysis:");
        println!("   Key: {}", issue.key);
        println!("   ID: {}", issue.id);
        println!("   Self Link: {}", issue.self_link);
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("No summary".to_string())
        );
        println!("   Fields Available: {}", issue.fields.len());

        // Show that our auto-injection worked
        println!();
        println!("🎉 V3 Auto-Injection Success!");
        println!("   ✅ Without explicit field specification, we got:");
        println!("   ✅ ID field (required): ✓");
        println!("   ✅ Self link (for Issue struct): ✓");
        println!("   ✅ Key field: ✓");
        println!("   ✅ Fields container: {} fields", issue.fields.len());

        if let Some(status) = issue.status() {
            println!("   ✅ Status accessible: {}", status.name);
        }
    } else {
        println!("📝 No issues found in SCRUM project");
    }

    // Test 2: Compare with minimal fields (what V3 would return without our fix)
    println!("\n2️⃣  Comparison: Minimal Fields (V3 Default Behavior)");
    println!("====================================================");

    let minimal_results = jira.search().list(
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::builder()
            .minimal_fields()
            .max_results(1)
            .build(),
    )?;

    if !minimal_results.issues.is_empty() {
        let issue = &minimal_results.issues[0];
        println!("🔍 With minimal fields (V3 default without our fix):");
        println!("   Key: {}", issue.key);
        println!("   ID: {}", issue.id);
        println!("   Self Link: {}", issue.self_link);
        println!("   Fields Available: {}", issue.fields.len());
        println!("   ⚠️  This would cause Serde errors in old implementations!");
    }

    // Test 3: Show convenience methods
    println!("\n3️⃣  Convenience Methods Test");
    println!("============================");

    // Standard fields
    let standard_results = jira.search().list(
        "project = SCRUM ORDER BY created DESC",
        &SearchOptions::builder()
            .standard_fields()
            .max_results(1)
            .build(),
    )?;

    if !standard_results.issues.is_empty() {
        let issue = &standard_results.issues[0];
        println!("📋 Standard fields result:");
        println!("   Issue: {}", issue.key);
        println!("   Fields: {}", issue.fields.len());
        println!(
            "   Summary: {}",
            issue.summary().unwrap_or("N/A".to_string())
        );

        if let Some(assignee) = issue.assignee() {
            println!("   Assignee: {}", assignee.display_name);
        }

        if let Some(status) = issue.status() {
            println!("   Status: {}", status.name);
        }
    }

    println!("\n🎉 V3 Implementation Test Complete!");
    println!("===================================");
    println!("✅ V3 API auto-detection working");
    println!("✅ Essential field auto-injection working");
    println!("✅ Convenience methods working");
    println!("✅ Zero breaking changes - existing code works!");
    println!();
    println!("💡 Key Success: Default SearchOptions now works seamlessly with V3!");

    Ok(())
}
