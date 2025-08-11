//! Example demonstrating issue relationship graph extraction and analysis
//!
//! This example shows how to:
//! 1. Extract relationships from a Jira issue to a specified depth
//! 2. Analyze the relationship graph
//! 3. Export to JSON for AI agent processing
//!
//! Run with: cargo run --example relationships

use gouqi::relationships::{GraphOptions, RelationshipGraph};
use gouqi::{Credentials, Jira};
use std::env;
use tracing::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing (this is optional - you may not have tracing_subscriber)
    // tracing_subscriber::fmt::init();

    // Check for required environment variables
    let host = env::var("JIRA_HOST").map_err(|_| "Missing JIRA_HOST environment variable")?;
    let issue_key = env::args().nth(1).unwrap_or_else(|| "DEMO-1".to_string());

    info!("Connecting to Jira at: {}", host);
    info!("Analyzing relationships for issue: {}", issue_key);

    // Create Jira client
    let credentials = if let (Ok(username), Ok(password)) =
        (env::var("JIRA_USERNAME"), env::var("JIRA_PASSWORD"))
    {
        Credentials::Basic(username, password)
    } else if let Ok(token) = env::var("JIRA_TOKEN") {
        Credentials::Bearer(token)
    } else {
        info!("No credentials provided, using anonymous access");
        Credentials::Anonymous
    };

    let jira = Jira::new(host, credentials)?;

    // Example 1: Extract all relationships to depth 2
    info!("Extracting all relationships to depth 2...");
    match jira.issues().get_relationship_graph(&issue_key, 2, None) {
        Ok(graph) => {
            print_graph_summary(&graph);

            // Export to JSON for AI agent processing
            let json = serde_json::to_string_pretty(&graph)?;
            println!("\n=== JSON Export (AI-friendly format) ===");
            println!("{}", json);
        }
        Err(e) => error!("Failed to extract relationship graph: {:?}", e),
    }

    // Example 2: Extract only blocking relationships
    info!("\nExtracting only blocking relationships...");
    let blocking_options = GraphOptions {
        include_types: Some(vec!["blocks".to_string(), "blocked_by".to_string()]),
        exclude_types: None,
        include_custom: false,
        bidirectional: true,
    };

    match jira
        .issues()
        .get_relationship_graph(&issue_key, 3, Some(blocking_options))
    {
        Ok(blocking_graph) => {
            print_graph_summary(&blocking_graph);
            analyze_blocking_chain(&blocking_graph, &issue_key);
        }
        Err(e) => error!("Failed to extract blocking relationships: {:?}", e),
    }

    // Example 3: Bulk relationship extraction
    info!("\nDemonstrating bulk relationship extraction...");
    let related_issues = vec![
        format!("{}", issue_key),
        format!("DEMO-2"),
        format!("DEMO-3"),
    ];

    match jira.issues().get_bulk_relationships(&related_issues, None) {
        Ok(bulk_graph) => {
            print_graph_summary(&bulk_graph);
        }
        Err(e) => error!("Failed to extract bulk relationships: {:?}", e),
    }

    Ok(())
}

fn print_graph_summary(graph: &RelationshipGraph) {
    println!("\n=== Relationship Graph Summary ===");
    println!("Source: {}", graph.metadata.source);
    println!("Issues: {}", graph.metadata.issue_count);
    println!("Total relationships: {}", graph.metadata.relationship_count);
    println!("Max depth: {}", graph.metadata.max_depth);
    println!("Generated: {}", graph.metadata.timestamp);

    if let Some(root) = &graph.metadata.root_issue {
        println!("Root issue: {}", root);
    }

    // Print detailed relationships
    for (issue_key, relationships) in &graph.issues {
        if !relationships.is_empty() {
            println!("\nIssue: {}", issue_key);

            if !relationships.blocks.is_empty() {
                println!("  Blocks: {}", relationships.blocks.join(", "));
            }
            if !relationships.blocked_by.is_empty() {
                println!("  Blocked by: {}", relationships.blocked_by.join(", "));
            }
            if !relationships.relates_to.is_empty() {
                println!("  Relates to: {}", relationships.relates_to.join(", "));
            }
            if let Some(parent) = &relationships.parent {
                println!("  Parent: {}", parent);
            }
            if !relationships.children.is_empty() {
                println!("  Children: {}", relationships.children.join(", "));
            }
            if let Some(epic) = &relationships.epic {
                println!("  Epic: {}", epic);
            }
            // Note: stories field doesn't exist in IssueRelationships
            // if !relationships.stories.is_empty() {
            //     println!("  Stories: {}", relationships.stories.join(", "));
            // }
            if !relationships.custom.is_empty() {
                for (custom_type, targets) in &relationships.custom {
                    println!("  {}: {}", custom_type, targets.join(", "));
                }
            }
        }
    }
}

fn analyze_blocking_chain(graph: &RelationshipGraph, root_issue: &str) {
    println!("\n=== Blocking Chain Analysis ===");

    // Find what this issue blocks (chain forward)
    if let Some(relationships) = graph.get_relationships(root_issue) {
        if !relationships.blocks.is_empty() {
            println!("{} blocks:", root_issue);
            for blocked_issue in &relationships.blocks {
                print_blocking_chain(graph, blocked_issue, 1);
            }
        }

        // Find what blocks this issue (chain backward)
        if !relationships.blocked_by.is_empty() {
            println!("{} is blocked by:", root_issue);
            for blocking_issue in &relationships.blocked_by {
                print_blocking_chain_reverse(graph, blocking_issue, 1);
            }
        }
    }

    // Check for circular dependencies
    if let Some(path) = graph.get_path(root_issue, root_issue) {
        if path.len() > 1 {
            println!("⚠️  Circular dependency detected: {}", path.join(" -> "));
        }
    }
}

fn print_blocking_chain(graph: &RelationshipGraph, issue: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}├─ {}", indent, issue);

    if let Some(relationships) = graph.get_relationships(issue) {
        for blocked_issue in &relationships.blocks {
            if depth < 5 {
                // Limit depth to avoid infinite loops
                print_blocking_chain(graph, blocked_issue, depth + 1);
            }
        }
    }
}

fn print_blocking_chain_reverse(graph: &RelationshipGraph, issue: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}├─ {}", indent, issue);

    if let Some(relationships) = graph.get_relationships(issue) {
        for blocking_issue in &relationships.blocked_by {
            if depth < 5 {
                // Limit depth to avoid infinite loops
                print_blocking_chain_reverse(graph, blocking_issue, depth + 1);
            }
        }
    }
}
