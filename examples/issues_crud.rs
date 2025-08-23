//! Issues CRUD Operations Example
//!
//! This example demonstrates the comprehensive CRUD operations available
//! for Jira issues, including individual operations, watcher management,
//! voting, and bulk operations.

use gouqi::issues::{BulkIssueUpdate, BulkUpdateRequest};
use gouqi::{Credentials, Jira, Result};
use std::collections::BTreeMap;
use std::env;

fn main() -> Result<()> {
    let jira_url =
        env::var("JIRA_URL").unwrap_or_else(|_| "https://your-instance.atlassian.net".to_string());
    let jira_user = env::var("JIRA_USER").expect("JIRA_USER environment variable required");
    let jira_token = env::var("JIRA_TOKEN").expect("JIRA_TOKEN environment variable required");

    // Create Jira client with basic authentication
    let credentials = Credentials::Basic(jira_user, jira_token);
    let jira = Jira::new(&jira_url, credentials)?;
    let issues = jira.issues();

    println!("🔧 Jira Issues CRUD Operations Demo");
    println!("===================================\n");

    // Example issue keys - replace with actual issue keys from your instance
    let issue_key = "DEMO-123";
    let username = "johndoe";

    // ============================================================================
    // INDIVIDUAL ISSUE OPERATIONS
    // ============================================================================

    println!("📝 Individual Issue Operations");
    println!("------------------------------");

    // Assign an issue to a user
    println!("🔄 Assigning issue {} to user {}...", issue_key, username);
    match issues.assign(issue_key, Some(username.to_string())) {
        Ok(_) => println!("✅ Issue assigned successfully"),
        Err(e) => println!("❌ Failed to assign issue: {}", e),
    }

    // Unassign an issue
    println!("🔄 Unassigning issue {}...", issue_key);
    match issues.assign(issue_key, None) {
        Ok(_) => println!("✅ Issue unassigned successfully"),
        Err(e) => println!("❌ Failed to unassign issue: {}", e),
    }

    // Archive an issue (if you have permission and the issue is appropriate)
    println!("🔄 Archiving issue {}...", issue_key);
    match issues.archive(issue_key) {
        Ok(_) => println!("✅ Issue archived successfully"),
        Err(e) => println!("❌ Failed to archive issue: {}", e),
    }

    // Delete an issue (CAUTION: This is permanent!)
    // Uncomment only if you want to actually delete the issue
    /*
    println!("🔄 Deleting issue {}...", issue_key);
    match issues.delete(issue_key) {
        Ok(_) => println!("✅ Issue deleted successfully"),
        Err(e) => println!("❌ Failed to delete issue: {}", e),
    }
    */

    println!();

    // ============================================================================
    // WATCHER MANAGEMENT
    // ============================================================================

    println!("👥 Watcher Management");
    println!("--------------------");

    // Get current watchers
    println!("🔄 Getting watchers for issue {}...", issue_key);
    match issues.get_watchers(issue_key) {
        Ok(watchers) => {
            println!(
                "✅ Found {} watchers, total watch count: {}",
                watchers.watchers.len(),
                watchers.watch_count
            );
            println!("   Currently watching: {}", watchers.is_watching);
            for watcher in &watchers.watchers {
                println!(
                    "   - {} ({})",
                    watcher.display_name,
                    watcher.name.as_deref().unwrap_or("no username")
                );
            }
        }
        Err(e) => println!("❌ Failed to get watchers: {}", e),
    }

    // Add a watcher
    println!("🔄 Adding watcher {} to issue {}...", username, issue_key);
    match issues.add_watcher(issue_key, username.to_string()) {
        Ok(_) => println!("✅ Watcher added successfully"),
        Err(e) => println!("❌ Failed to add watcher: {}", e),
    }

    // Remove a watcher
    println!(
        "🔄 Removing watcher {} from issue {}...",
        username, issue_key
    );
    match issues.remove_watcher(issue_key, username.to_string()) {
        Ok(_) => println!("✅ Watcher removed successfully"),
        Err(e) => println!("❌ Failed to remove watcher: {}", e),
    }

    println!();

    // ============================================================================
    // VOTING OPERATIONS
    // ============================================================================

    println!("👍 Voting Operations");
    println!("-------------------");

    // Vote for an issue
    println!("🔄 Voting for issue {}...", issue_key);
    match issues.vote(issue_key) {
        Ok(_) => println!("✅ Vote cast successfully"),
        Err(e) => println!("❌ Failed to vote: {}", e),
    }

    // Remove vote from an issue
    println!("🔄 Removing vote from issue {}...", issue_key);
    match issues.unvote(issue_key) {
        Ok(_) => println!("✅ Vote removed successfully"),
        Err(e) => println!("❌ Failed to remove vote: {}", e),
    }

    println!();

    // ============================================================================
    // BULK OPERATIONS
    // ============================================================================

    println!("📦 Bulk Operations");
    println!("-----------------");

    // Bulk update multiple issues
    println!("🔄 Performing bulk update on issues...");

    // Create bulk update request
    let mut fields1 = BTreeMap::new();
    fields1.insert(
        "summary".to_string(),
        serde_json::Value::String("Updated via bulk operation".to_string()),
    );

    let mut fields2 = BTreeMap::new();
    fields2.insert(
        "priority".to_string(),
        serde_json::json!({ "name": "High" }),
    );

    let bulk_request = BulkUpdateRequest {
        issue_updates: vec![
            BulkIssueUpdate {
                key: "DEMO-123".to_string(),
                fields: fields1,
            },
            BulkIssueUpdate {
                key: "DEMO-124".to_string(),
                fields: fields2,
            },
        ],
    };

    match issues.bulk_update(bulk_request) {
        Ok(response) => {
            println!("✅ Bulk update completed");
            println!("   Updated {} issues", response.issues.len());
            if !response.errors.is_empty() {
                println!("   {} errors occurred:", response.errors.len());
                for error in &response.errors {
                    println!("   - Error: {:?}", error);
                }
            }
        }
        Err(e) => println!("❌ Failed to perform bulk update: {}", e),
    }

    println!();
    println!("🎉 Issues CRUD operations demo completed!");
    println!("\nNote: This example uses placeholder issue keys and usernames.");
    println!("Replace them with actual values from your Jira instance.");
    println!("Set environment variables: JIRA_URL, JIRA_USER, JIRA_TOKEN");

    Ok(())
}
