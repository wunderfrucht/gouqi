//! 📝 V3 Test Data Creator - Creates multiple issues for testing pagination
//!
//! This creates several test issues in the SCRUM project so we can properly test:
//! - Multi-page pagination scenarios
//! - nextPageToken functionality
//! - Iterator performance with multiple pages
//! - Total count estimation accuracy
//!
//! Run with: INTEGRATION_JIRA_TOKEN=token JIRA_USER=email cargo run --example v3_create_test_data

use gouqi::{Credentials, Jira};
use serde_json::json;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("INTEGRATION_JIRA_TOKEN")
        .expect("Please set INTEGRATION_JIRA_TOKEN environment variable");

    let user =
        env::var("JIRA_USER").expect("Please set JIRA_USER environment variable (your email)");

    println!("📝 V3 Test Data Creator");
    println!("======================");

    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(user, token),
    )?;

    println!("🌐 API Version: {:?}", jira.get_search_api_version());
    println!("📡 Host: https://gouji.atlassian.net");
    println!();

    // Get current issue count
    let current = jira.search().list(
        "project = SCRUM AND summary ~ 'V3Test*' ORDER BY created DESC",
        &gouqi::SearchOptions::builder().max_results(1).build(),
    )?;

    println!("📊 Current test issues in SCRUM: {}", current.total);
    println!();

    // Create several test issues for pagination testing
    let issues_to_create = 100; // This will give us multiple pages for testing
    println!(
        "🏗️  Creating {} test issues for V3 pagination testing...",
        issues_to_create
    );

    for i in 1..=issues_to_create {
        println!("📝 Creating test issue {} of {}...", i, issues_to_create);

        let issue_data = json!({
            "project": {
                "key": "SCRUM"
            },
            "summary": format!("V3Test-{:03} - Pagination Test Issue", i),
            "description": format!("Test issue #{} created for V3 API pagination testing.\n\nThis issue helps validate:\n- nextPageToken functionality\n- Iterator performance\n- Total count estimation\n- maxResults enforcement", i),
            "issuetype": {
                "name": "Story"
            }
        });

        let custom_issue = gouqi::issues::CreateCustomIssue { fields: issue_data };

        match jira.issues().create_from_custom_issue(custom_issue) {
            Ok(created) => {
                println!("   ✅ Created: {}", created.key);
            }
            Err(e) => {
                println!("   ❌ Failed to create issue {}: {:?}", i, e);
                println!("   🤔 This might be due to project permissions or configuration");
            }
        }

        // Small delay to avoid rate limiting
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    println!();
    println!("✅ Test data creation completed!");
    println!();

    // Verify the test data
    println!("🔍 Verifying test data...");
    let verification = jira.search().list(
        "project = SCRUM AND summary ~ 'V3Test*' ORDER BY created DESC",
        &gouqi::SearchOptions::builder().max_results(50).build(),
    )?;

    println!("📊 Total V3Test issues found: {}", verification.total);
    println!("📋 Issues in this page: {}", verification.issues.len());
    println!("📄 Is last page: {:?}", verification.is_last_page);

    if !verification.issues.is_empty() {
        println!("🎯 Sample issues created:");
        for (idx, issue) in verification.issues.iter().take(5).enumerate() {
            println!(
                "   {}. {} - {}",
                idx + 1,
                issue.key,
                issue.summary().unwrap_or("No summary".to_string())
            );
        }

        if verification.issues.len() > 5 {
            println!("   ... and {} more", verification.issues.len() - 5);
        }
    }

    println!();
    println!("🧪 Ready for comprehensive V3 pagination testing!");
    println!("   Run: cargo run --example v3_exhaustive_test");
    println!();
    println!(
        "🗑️  To cleanup test data later, search for 'summary ~ \"V3Test*\"' and delete the issues"
    );

    Ok(())
}
