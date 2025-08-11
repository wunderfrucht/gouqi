//! Async example demonstrating issue relationship graph extraction and analysis
//!
//! This example shows how to:
//! 1. Extract relationships from a Jira issue asynchronously to a specified depth
//! 2. Use concurrent processing for better performance
//! 3. Analyze the relationship graph
//! 4. Export to JSON for AI agent processing
//!
//! Run with: cargo run --example async_relationships --features async

#[cfg(feature = "async")]
use gouqi::{Credentials, r#async::Jira};
#[cfg(feature = "async")]
use gouqi::relationships::{RelationshipGraph, GraphOptions};
#[cfg(feature = "async")]
use std::env;
#[cfg(feature = "async")]
use tracing::{info, error};
#[cfg(feature = "async")]
use tokio::time::Instant;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing (optional)
    // tracing_subscriber::fmt::init();

    // Check for required environment variables
    let host = env::var("JIRA_HOST").map_err(|_| "Missing JIRA_HOST environment variable")?;
    let issue_key = env::args()
        .nth(1)
        .unwrap_or_else(|| "DEMO-1".to_string());
    
    info!("Connecting to Jira at: {}", host);
    info!("Analyzing relationships for issue: {}", issue_key);

    // Create async Jira client
    let credentials = if let (Ok(username), Ok(password)) = 
        (env::var("JIRA_USERNAME"), env::var("JIRA_PASSWORD")) {
        Credentials::Basic(username, password)
    } else if let Ok(token) = env::var("JIRA_TOKEN") {
        Credentials::Bearer(token)
    } else {
        info!("No credentials provided, using anonymous access");
        Credentials::Anonymous
    };

    let jira = Jira::new(host, credentials)?;

    // Example 1: Extract all relationships to depth 2 (async)
    info!("Extracting all relationships to depth 2 (async)...");
    let start = Instant::now();
    
    match jira.issues().get_relationship_graph(&issue_key, 2, None).await {
        Ok(graph) => {
            let duration = start.elapsed();
            print_graph_summary(&graph);
            info!("Async traversal completed in {:?}", duration);
            
            // Export to JSON for AI agent processing
            let json = serde_json::to_string_pretty(&graph)?;
            println!("\n=== JSON Export (AI-friendly format) ===");
            println!("{}", json);
        }
        Err(e) => error!("Failed to extract relationship graph: {:?}", e),
    }

    // Example 2: Extract only blocking relationships with options
    info!("\nExtracting only blocking relationships (async)...");
    let blocking_options = GraphOptions {
        include_types: Some(vec!["blocks".to_string(), "blocked_by".to_string()]),
        exclude_types: None,
        include_custom: false,
        bidirectional: true,
    };

    match jira.issues().get_relationship_graph(&issue_key, 3, Some(blocking_options)).await {
        Ok(blocking_graph) => {
            print_graph_summary(&blocking_graph);
            analyze_blocking_chain(&blocking_graph, &issue_key);
        }
        Err(e) => error!("Failed to extract blocking relationships: {:?}", e),
    }

    // Example 3: Concurrent bulk relationship extraction
    info!("\nDemonstrating concurrent bulk relationship extraction...");
    let related_issues = vec![
        format!("{}", issue_key),
        format!("DEMO-2"),
        format!("DEMO-3"),
        format!("DEMO-4"),
        format!("DEMO-5"),
    ];

    let start = Instant::now();
    match jira.issues().get_bulk_relationships(&related_issues, None).await {
        Ok(bulk_graph) => {
            let duration = start.elapsed();
            print_graph_summary(&bulk_graph);
            info!("Concurrent bulk extraction completed in {:?}", duration);
        }
        Err(e) => error!("Failed to extract bulk relationships: {:?}", e),
    }

    // Example 4: Performance comparison - sequential vs concurrent
    info!("\nPerformance comparison: Sequential vs Concurrent...");
    await_performance_comparison(&jira, &related_issues).await?;

    // Example 5: Complex relationship analysis
    info!("\nComplex relationship analysis...");
    await_complex_analysis(&jira, &issue_key).await?;

    Ok(())
}

#[cfg(feature = "async")]
async fn await_performance_comparison(
    jira: &Jira, 
    issue_keys: &[String]
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Performance Comparison ===");
    
    // Sequential approach
    let start = Instant::now();
    let mut sequential_graphs = Vec::new();
    for issue_key in issue_keys {
        match jira.issues().get_relationship_graph(issue_key, 1, None).await {
            Ok(graph) => sequential_graphs.push(graph),
            Err(_) => continue,
        }
    }
    let sequential_duration = start.elapsed();
    
    // Concurrent approach (bulk)
    let start = Instant::now();
    let concurrent_result = jira.issues().get_bulk_relationships(issue_keys, None).await;
    let concurrent_duration = start.elapsed();
    
    println!("Sequential processing: {:?} ({} graphs)", sequential_duration, sequential_graphs.len());
    match concurrent_result {
        Ok(graph) => {
            println!("Concurrent processing: {:?} ({} issues)", concurrent_duration, graph.metadata.issue_count);
            
            if concurrent_duration < sequential_duration {
                let speedup = sequential_duration.as_millis() as f64 / concurrent_duration.as_millis() as f64;
                println!("🚀 Concurrent approach is {:.2}x faster!", speedup);
            }
        }
        Err(e) => error!("Concurrent processing failed: {:?}", e),
    }
    
    Ok(())
}

#[cfg(feature = "async")]
async fn await_complex_analysis(
    jira: &Jira, 
    root_issue: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Complex Relationship Analysis ===");
    
    // Get deep relationship graph
    match jira.issues().get_relationship_graph(root_issue, 3, None).await {
        Ok(graph) => {
            // Find all unique relationship types
            let mut relationship_types = std::collections::HashSet::new();
            for relationships in graph.issues.values() {
                if !relationships.blocks.is_empty() { relationship_types.insert("blocks"); }
                if !relationships.blocked_by.is_empty() { relationship_types.insert("blocked_by"); }
                if !relationships.relates_to.is_empty() { relationship_types.insert("relates_to"); }
                if !relationships.duplicates.is_empty() { relationship_types.insert("duplicates"); }
                if relationships.parent.is_some() { relationship_types.insert("parent"); }
                if !relationships.children.is_empty() { relationship_types.insert("children"); }
                if relationships.epic.is_some() { relationship_types.insert("epic"); }
                if let Some(_custom_type) = relationships.custom.keys().next() {
                    relationship_types.insert("custom");
                }
            }
            
            println!("Relationship types found: {:?}", relationship_types);
            
            // Find longest path from root
            let mut max_distance = 0;
            let mut farthest_issue = root_issue.to_string();
            
            for issue_key in graph.get_issue_keys() {
                if let Some(path) = graph.get_path(root_issue, issue_key) {
                    if path.len() > max_distance {
                        max_distance = path.len();
                        farthest_issue = issue_key.clone();
                    }
                }
            }
            
            println!("Farthest connected issue: {} (distance: {})", farthest_issue, max_distance - 1);
            
            // Find all cycles
            detect_cycles(&graph, root_issue);
            
            // Generate summary statistics
            let total_relationships: usize = graph.issues.values()
                .map(|rel| {
                    rel.blocks.len() + rel.blocked_by.len() + rel.relates_to.len() + 
                    rel.duplicates.len() + rel.children.len() +
                    (if rel.parent.is_some() { 1 } else { 0 }) +
                    (if rel.epic.is_some() { 1 } else { 0 }) +
                    rel.custom.values().map(|v| v.len()).sum::<usize>()
                })
                .sum();
                
            println!("Graph density: {:.2} relationships per issue", 
                total_relationships as f64 / graph.metadata.issue_count as f64);
        }
        Err(e) => error!("Failed to perform complex analysis: {:?}", e),
    }
    
    Ok(())
}

#[cfg(feature = "async")]
fn detect_cycles(graph: &RelationshipGraph, _root_issue: &str) {
    println!("\n--- Cycle Detection ---");
    
    // Check for blocking cycles
    for issue_key in graph.get_issue_keys() {
        if let Some(path) = graph.get_path(issue_key, issue_key) {
            if path.len() > 1 {
                println!("⚠️  Circular dependency detected: {}", path.join(" -> "));
            }
        }
    }
    
    // Check for parent-child cycles (these would be problematic)
    fn has_parent_cycle(graph: &RelationshipGraph, issue: &str, visited: &mut std::collections::HashSet<String>) -> bool {
        if visited.contains(issue) {
            return true;
        }
        
        visited.insert(issue.to_string());
        
        if let Some(relationships) = graph.get_relationships(issue) {
            if let Some(parent) = &relationships.parent {
                if has_parent_cycle(graph, parent, visited) {
                    return true;
                }
            }
        }
        
        visited.remove(issue);
        false
    }
    
    for issue_key in graph.get_issue_keys() {
        let mut visited = std::collections::HashSet::new();
        if has_parent_cycle(graph, issue_key, &mut visited) {
            println!("🚨 Parent-child cycle detected involving: {}", issue_key);
        }
    }
}

#[cfg(feature = "async")]
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

    // Print detailed relationships for first few issues
    let mut count = 0;
    for (issue_key, relationships) in &graph.issues {
        if count >= 3 { // Limit output for readability
            println!("... and {} more issues", graph.metadata.issue_count - count);
            break;
        }
        
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
            if !relationships.custom.is_empty() {
                for (custom_type, targets) in &relationships.custom {
                    println!("  {}: {}", custom_type, targets.join(", "));
                }
            }
            count += 1;
        }
    }
}

#[cfg(feature = "async")]
fn analyze_blocking_chain(graph: &RelationshipGraph, root_issue: &str) {
    println!("\n=== Blocking Chain Analysis ===");
    
    if let Some(relationships) = graph.get_relationships(root_issue) {
        if !relationships.blocks.is_empty() {
            println!("{} blocks:", root_issue);
            for blocked_issue in &relationships.blocks {
                print_blocking_chain(graph, blocked_issue, 1);
            }
        }
        
        if !relationships.blocked_by.is_empty() {
            println!("{} is blocked by:", root_issue);
            for blocking_issue in &relationships.blocked_by {
                print_blocking_chain_reverse(graph, blocking_issue, 1);
            }
        }
    }
}

#[cfg(feature = "async")]
fn print_blocking_chain(graph: &RelationshipGraph, issue: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}├─ {}", indent, issue);
    
    if let Some(relationships) = graph.get_relationships(issue) {
        for blocked_issue in &relationships.blocks {
            if depth < 5 {
                print_blocking_chain(graph, blocked_issue, depth + 1);
            }
        }
    }
}

#[cfg(feature = "async")]
fn print_blocking_chain_reverse(graph: &RelationshipGraph, issue: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}├─ {}", indent, issue);
    
    if let Some(relationships) = graph.get_relationships(issue) {
        for blocking_issue in &relationships.blocked_by {
            if depth < 5 {
                print_blocking_chain_reverse(graph, blocking_issue, depth + 1);
            }
        }
    }
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --example async_relationships --features async");
    std::process::exit(1);
}