//! Async MCP (Model Context Protocol) Integration Examples
//!
//! This example demonstrates how to use the MCP utilities with the async Jira client
//! to integrate Jira data with MCP-compatible systems in an asynchronous context.

#[cfg(feature = "async")]
use gouqi::mcp::{MCPError, MCPResource};
#[cfg(feature = "async")]
use gouqi::mcp::{error, uri, validation};
#[cfg(feature = "async")]
use gouqi::{Credentials, r#async::Jira};
#[cfg(feature = "async")]
use std::collections::HashMap;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize async Jira client
    let jira = Jira::new(
        "https://jira.example.com",
        Credentials::Basic("username".to_string(), "password".to_string()),
    )?;

    println!("=== Async MCP Integration Examples ===");
    println!("This example showcases comprehensive MCP integration including:");
    println!("- Extended entity support (User, Version, Board, Sprint, Component)");
    println!("- Additional tool operations (transitions, attachments, components)");
    println!("- Concurrent processing and error handling");
    println!();

    // Demonstrate async MCP operations
    demonstrate_async_mcp_operations(&jira).await?;

    // Demonstrate async error handling
    demonstrate_async_error_handling(&jira).await?;

    // Demonstrate bulk resource processing
    demonstrate_bulk_resource_processing(&jira).await?;

    Ok(())
}

#[cfg(feature = "async")]
async fn demonstrate_async_mcp_operations(_jira: &Jira) -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstrating async MCP operations...");

    // Example of how you would search for issues and convert them to MCP resources
    println!("\n1. Async Issue Search and MCP Conversion:");
    println!("   In a real application, you would:");
    println!(
        "   - Use: let results = jira.search().list(\"project = DEMO\", &Default::default()).await?"
    );
    println!("   - Convert each issue: issue.to_mcp_resource(\"https://jira.example.com\")");

    // Simulate the process without making actual API calls
    let example_mcp_resources = simulate_issue_search_to_mcp().await;
    println!(
        "   Simulated results: {} MCP resources created",
        example_mcp_resources.len()
    );

    for (i, resource) in example_mcp_resources.iter().enumerate() {
        println!(
            "   Resource {}: {} ({})",
            i + 1,
            resource.name,
            resource.uri
        );
    }

    println!("\n2. Async Project Listing and MCP Conversion:");
    println!("   In a real application, you would:");
    println!("   - Use: let projects = jira.projects().iter().await?");
    println!("   - Convert each project: project.to_mcp_resource(\"https://jira.example.com\")");

    let example_project_resources = simulate_project_list_to_mcp().await;
    println!(
        "   Simulated results: {} project MCP resources created",
        example_project_resources.len()
    );

    for (i, resource) in example_project_resources.iter().enumerate() {
        println!("   Project {}: {} ({})", i + 1, resource.name, resource.uri);
    }

    println!("\n3. Extended Entity MCP Support:");
    println!("   New in this version - convert additional entities to MCP resources:");

    // Demonstrate User entity conversion
    let example_user_resources = simulate_user_list_to_mcp().await;
    println!("   Users: {} MCP resources", example_user_resources.len());
    for resource in &example_user_resources {
        println!("     - {} ({})", resource.name, resource.uri);
    }

    // Demonstrate Version entity conversion
    let example_version_resources = simulate_version_list_to_mcp().await;
    println!(
        "   Versions: {} MCP resources",
        example_version_resources.len()
    );
    for resource in &example_version_resources {
        println!("     - {} ({})", resource.name, resource.uri);
    }

    // Demonstrate Board entity conversion
    let example_board_resources = simulate_board_list_to_mcp().await;
    println!("   Boards: {} MCP resources", example_board_resources.len());
    for resource in &example_board_resources {
        println!("     - {} ({})", resource.name, resource.uri);
    }

    // Demonstrate Sprint entity conversion
    let example_sprint_resources = simulate_sprint_list_to_mcp().await;
    println!(
        "   Sprints: {} MCP resources",
        example_sprint_resources.len()
    );
    for resource in &example_sprint_resources {
        println!("     - {} ({})", resource.name, resource.uri);
    }

    // Demonstrate Component entity conversion
    let example_component_resources = simulate_component_list_to_mcp().await;
    println!(
        "   Components: {} MCP resources",
        example_component_resources.len()
    );
    for resource in &example_component_resources {
        println!("     - {} ({})", resource.name, resource.uri);
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn demonstrate_async_error_handling(_jira: &Jira) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Async Error Handling ===");

    // Simulate various error scenarios and their MCP conversion
    println!("Simulating async error scenarios...");

    let error_scenarios = vec![
        ("Network timeout", gouqi::Error::Unauthorized),
        ("Resource not found", gouqi::Error::NotFound),
        ("Method not allowed", gouqi::Error::MethodNotAllowed),
    ];

    for (scenario, jira_error) in error_scenarios {
        println!("\nScenario: {}", scenario);
        let mcp_error = error::to_mcp_error(&jira_error);
        println!(
            "  MCP Error: code={}, message=\"{}\"",
            mcp_error.code, mcp_error.message
        );

        // In a real async handler, you would return this error
        if let Some(data) = &mcp_error.data {
            println!("  Error data: {}", serde_json::to_string_pretty(data)?);
        }
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn demonstrate_bulk_resource_processing(
    _jira: &Jira,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Bulk Resource Processing ===");

    // Demonstrate processing multiple resources concurrently
    println!("Demonstrating concurrent resource processing...");
    println!("This includes the new extended entity support for comprehensive MCP integration.");

    // Simulate processing multiple issue keys concurrently
    let issue_keys = vec!["DEMO-123", "DEMO-124", "DEMO-125", "PROJ-001", "PROJ-002"];

    println!("Processing {} issues concurrently...", issue_keys.len());

    // In a real implementation, you would use futures::future::join_all
    // to process multiple issues concurrently:
    // let futures: Vec<_> = issue_keys.iter().map(|key| async {
    //     match jira.issues().get(key).await {
    //         Ok(issue) => Ok(issue.to_mcp_resource("https://jira.example.com")),
    //         Err(e) => Err(error::to_mcp_error(&e))
    //     }
    // }).collect();
    // let results = futures::future::join_all(futures).await;

    // For this example, we'll simulate the results
    let simulated_results = simulate_concurrent_processing(issue_keys).await;

    println!("Results:");
    for (i, result) in simulated_results.iter().enumerate() {
        match result {
            Ok(resource) => println!("  ✓ Issue {}: {} -> {}", i + 1, resource.name, resource.uri),
            Err(error) => println!(
                "  ✗ Issue {}: Error {} - {}",
                i + 1,
                error.code,
                error.message
            ),
        }
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn simulate_issue_search_to_mcp() -> Vec<MCPResource> {
    // Simulate creating MCP resources from search results
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Simulate async work

    vec![
        MCPResource {
            uri: uri::issue_uri("DEMO-123"),
            name: "DEMO-123: Fix login bug".to_string(),
            description: Some("Users are unable to log in with their credentials".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project".to_string(), serde_json::json!("DEMO"));
                annotations.insert("status".to_string(), serde_json::json!("In Progress"));
                annotations.insert("issue_type".to_string(), serde_json::json!("Bug"));
                annotations
            }),
        },
        MCPResource {
            uri: uri::issue_uri("DEMO-124"),
            name: "DEMO-124: Add user registration".to_string(),
            description: Some("Implement user registration functionality".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project".to_string(), serde_json::json!("DEMO"));
                annotations.insert("status".to_string(), serde_json::json!("To Do"));
                annotations.insert("issue_type".to_string(), serde_json::json!("Feature"));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_project_list_to_mcp() -> Vec<MCPResource> {
    // Simulate creating MCP resources from project list
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await; // Simulate async work

    vec![
        MCPResource {
            uri: uri::project_uri("DEMO"),
            name: "DEMO: Demo Project".to_string(),
            description: Some("A demo project for testing".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project_type".to_string(), serde_json::json!("software"));
                annotations.insert("lead".to_string(), serde_json::json!("Project Lead"));
                annotations
            }),
        },
        MCPResource {
            uri: uri::project_uri("PROJ"),
            name: "PROJ: Main Project".to_string(),
            description: Some("The main project for the application".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project_type".to_string(), serde_json::json!("software"));
                annotations.insert("lead".to_string(), serde_json::json!("Tech Lead"));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_concurrent_processing(
    issue_keys: Vec<&str>,
) -> Vec<Result<MCPResource, MCPError>> {
    // Simulate processing multiple issues concurrently
    use futures::future::join_all;

    let futures: Vec<_> = issue_keys
        .into_iter()
        .map(|key| async move {
            // Simulate async work with varying delay based on key
            let delay_ms = (key.len() % 5 + 1) as u64 * 10; // 10-50ms based on key length
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            // Simulate some failures for demonstration
            if key.ends_with("125") {
                return Err(MCPError {
                    code: 404,
                    message: format!("Issue {} not found", key),
                    data: Some(serde_json::json!({
                        "type": "not_found_error",
                        "issue_key": key
                    })),
                });
            }

            // Simulate successful conversion
            Ok(MCPResource {
                uri: uri::issue_uri(key),
                name: format!("{}: Simulated issue", key),
                description: Some(format!("This is a simulated issue for {}", key)),
                mime_type: "application/json".to_string(),
                annotations: Some({
                    let mut annotations = HashMap::new();
                    annotations.insert(
                        "project".to_string(),
                        serde_json::json!(key.split('-').next().unwrap_or("UNKNOWN")),
                    );
                    annotations.insert("status".to_string(), serde_json::json!("Open"));
                    annotations.insert("issue_type".to_string(), serde_json::json!("Task"));
                    annotations
                }),
            })
        })
        .collect();

    join_all(futures).await
}

#[cfg(feature = "async")]
async fn simulate_user_list_to_mcp() -> Vec<MCPResource> {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    vec![
        MCPResource {
            uri: uri::user_uri("john.doe"),
            name: "User: John Doe".to_string(),
            description: Some("Active Jira user - john.doe".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("active".to_string(), serde_json::json!(true));
                annotations.insert(
                    "email".to_string(),
                    serde_json::json!("john.doe@example.com"),
                );
                annotations.insert(
                    "timezone".to_string(),
                    serde_json::json!("America/New_York"),
                );
                annotations
            }),
        },
        MCPResource {
            uri: uri::user_uri("jane.smith"),
            name: "User: Jane Smith".to_string(),
            description: Some("Active Jira user - jane.smith".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("active".to_string(), serde_json::json!(true));
                annotations.insert(
                    "email".to_string(),
                    serde_json::json!("jane.smith@example.com"),
                );
                annotations.insert("timezone".to_string(), serde_json::json!("UTC"));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_version_list_to_mcp() -> Vec<MCPResource> {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    vec![
        MCPResource {
            uri: uri::version_uri("10001"),
            name: "Version: v1.0.0".to_string(),
            description: Some("Project version v1.0.0".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("released".to_string(), serde_json::json!(true));
                annotations.insert("archived".to_string(), serde_json::json!(false));
                annotations.insert("release_date".to_string(), serde_json::json!("2024-01-15"));
                annotations
            }),
        },
        MCPResource {
            uri: uri::version_uri("10002"),
            name: "Version: v1.1.0".to_string(),
            description: Some("Project version v1.1.0".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("released".to_string(), serde_json::json!(false));
                annotations.insert("archived".to_string(), serde_json::json!(false));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_board_list_to_mcp() -> Vec<MCPResource> {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    vec![
        MCPResource {
            uri: uri::board_uri("1"),
            name: "Board: DEMO Kanban Board".to_string(),
            description: Some("Kanban board for DEMO project".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("type".to_string(), serde_json::json!("kanban"));
                annotations.insert("project_key".to_string(), serde_json::json!("DEMO"));
                annotations
            }),
        },
        MCPResource {
            uri: uri::board_uri("2"),
            name: "Board: PROJ Scrum Board".to_string(),
            description: Some("Scrum board for PROJ project".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("type".to_string(), serde_json::json!("scrum"));
                annotations.insert("project_key".to_string(), serde_json::json!("PROJ"));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_sprint_list_to_mcp() -> Vec<MCPResource> {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    vec![
        MCPResource {
            uri: uri::sprint_uri("1"),
            name: "Sprint: Sprint 1".to_string(),
            description: Some("Active sprint for development work".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("state".to_string(), serde_json::json!("active"));
                annotations.insert("origin_board_id".to_string(), serde_json::json!(1));
                annotations.insert(
                    "start_date".to_string(),
                    serde_json::json!("2024-01-01T00:00:00Z"),
                );
                annotations.insert(
                    "end_date".to_string(),
                    serde_json::json!("2024-01-14T23:59:59Z"),
                );
                annotations
            }),
        },
        MCPResource {
            uri: uri::sprint_uri("2"),
            name: "Sprint: Sprint 2".to_string(),
            description: Some("Future sprint for planning".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("state".to_string(), serde_json::json!("future"));
                annotations.insert("origin_board_id".to_string(), serde_json::json!(1));
                annotations
            }),
        },
    ]
}

#[cfg(feature = "async")]
async fn simulate_component_list_to_mcp() -> Vec<MCPResource> {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    vec![
        MCPResource {
            uri: uri::component_uri("10001"),
            name: "Component: Frontend".to_string(),
            description: Some("Frontend application component".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project_key".to_string(), serde_json::json!("DEMO"));
                annotations.insert("lead".to_string(), serde_json::json!("john.doe"));
                annotations
            }),
        },
        MCPResource {
            uri: uri::component_uri("10002"),
            name: "Component: Backend API".to_string(),
            description: Some("Backend API component".to_string()),
            mime_type: "application/json".to_string(),
            annotations: Some({
                let mut annotations = HashMap::new();
                annotations.insert("project_key".to_string(), serde_json::json!("DEMO"));
                annotations.insert("lead".to_string(), serde_json::json!("jane.smith"));
                annotations
            }),
        },
    ]
}

/// Example async MCP server handler
#[cfg(feature = "async")]
#[allow(dead_code)]
async fn example_async_mcp_handler(
    _jira: &Jira,
    tool_name: &str,
    input: serde_json::Value,
) -> Result<serde_json::Value, MCPError> {
    match tool_name {
        "jira_search_issues" => {
            // Extract and validate input
            let jql = input
                .get("jql")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: jql".to_string(),
                    data: None,
                })?;

            // Validate JQL
            validation::validate_jql(jql).map_err(|e| error::to_mcp_error(&e))?;

            // Extract optional pagination parameters
            let start_at = input
                .get("start_at")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let max_results = input
                .get("max_results")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);

            // Validate pagination
            validation::validate_pagination(start_at, max_results)
                .map_err(|e| error::to_mcp_error(&e))?;

            // In a real implementation, you would:
            // let mut options = SearchOptions::default();
            // if let Some(start) = start_at { options.start_at = Some(start); }
            // if let Some(max) = max_results { options.max_results = Some(max); }
            // let results = jira.search().list(jql, &options).await.map_err(|e| error::to_mcp_error(&e))?;
            // let mcp_resources: Vec<MCPResource> = results.issues.into_iter()
            //     .map(|issue| issue.to_mcp_resource("https://jira.example.com"))
            //     .collect();

            Ok(serde_json::json!({
                "message": "Would search for issues with JQL (async)",
                "jql": jql,
                "start_at": start_at,
                "max_results": max_results
            }))
        }
        "jira_list_issue_transitions" => {
            let issue_key = input
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: issue_key".to_string(),
                    data: None,
                })?;

            validation::validate_issue_key(issue_key).map_err(|e| error::to_mcp_error(&e))?;

            Ok(serde_json::json!({
                "message": "Would list transitions for issue (async)",
                "issue_key": issue_key
            }))
        }
        "jira_trigger_issue_transition" => {
            let issue_key = input
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: issue_key".to_string(),
                    data: None,
                })?;

            let transition_id = input
                .get("transition_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: transition_id".to_string(),
                    data: None,
                })?;

            validation::validate_issue_key(issue_key).map_err(|e| error::to_mcp_error(&e))?;

            Ok(serde_json::json!({
                "message": "Would trigger transition for issue (async)",
                "issue_key": issue_key,
                "transition_id": transition_id
            }))
        }
        "jira_list_issue_attachments" => {
            let issue_key = input
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: issue_key".to_string(),
                    data: None,
                })?;

            validation::validate_issue_key(issue_key).map_err(|e| error::to_mcp_error(&e))?;

            Ok(serde_json::json!({
                "message": "Would list attachments for issue (async)",
                "issue_key": issue_key
            }))
        }
        "jira_upload_issue_attachment" => {
            let issue_key = input
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: issue_key".to_string(),
                    data: None,
                })?;

            let filename = input
                .get("filename")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: filename".to_string(),
                    data: None,
                })?;

            validation::validate_issue_key(issue_key).map_err(|e| error::to_mcp_error(&e))?;

            Ok(serde_json::json!({
                "message": "Would upload attachment to issue (async)",
                "issue_key": issue_key,
                "filename": filename
            }))
        }
        "jira_create_project_component" => {
            let project_key = input
                .get("project_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: project_key".to_string(),
                    data: None,
                })?;

            let name = input
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: name".to_string(),
                    data: None,
                })?;

            validation::validate_project_key(project_key).map_err(|e| error::to_mcp_error(&e))?;

            Ok(serde_json::json!({
                "message": "Would create component in project (async)",
                "project_key": project_key,
                "name": name
            }))
        }
        "jira_update_project_component" => {
            let component_id = input
                .get("component_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: component_id".to_string(),
                    data: None,
                })?;

            Ok(serde_json::json!({
                "message": "Would update component (async)",
                "component_id": component_id
            }))
        }
        "jira_get_issue" => {
            // Extract and validate issue key
            let issue_key = input
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MCPError {
                    code: 400,
                    message: "Missing required parameter: issue_key".to_string(),
                    data: None,
                })?;

            // Validate issue key format
            validation::validate_issue_key(issue_key).map_err(|e| error::to_mcp_error(&e))?;

            // In a real implementation, you would:
            // let issue = jira.issues().get(issue_key).await.map_err(|e| error::to_mcp_error(&e))?;
            // let mcp_resource = issue.to_mcp_resource("https://jira.example.com");

            Ok(serde_json::json!({
                "message": "Would get issue (async)",
                "issue_key": issue_key
            }))
        }
        _ => Err(MCPError {
            code: 400,
            message: format!("Unknown tool: {}", tool_name),
            data: Some(serde_json::json!({
                "type": "unknown_tool_error",
                "tool_name": tool_name
            })),
        }),
    }
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example async_mcp_integration --features async");
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulate_issue_search_to_mcp() {
        let resources = simulate_issue_search_to_mcp().await;
        assert_eq!(resources.len(), 2);
        assert!(resources[0].name.contains("DEMO-123"));
        assert!(resources[1].name.contains("DEMO-124"));
    }

    #[tokio::test]
    async fn test_simulate_project_list_to_mcp() {
        let resources = simulate_project_list_to_mcp().await;
        assert_eq!(resources.len(), 2);
        assert!(resources[0].uri.contains("DEMO"));
        assert!(resources[1].uri.contains("PROJ"));
    }

    #[tokio::test]
    async fn test_simulate_concurrent_processing() {
        let issue_keys = vec!["DEMO-123", "DEMO-124", "DEMO-125"];
        let results = simulate_concurrent_processing(issue_keys).await;

        assert_eq!(results.len(), 3);

        // DEMO-125 should fail (simulated failure)
        assert!(results[2].is_err());

        // Others should succeed
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[tokio::test]
    async fn test_example_async_mcp_handler() {
        let jira = Jira::new("https://jira.example.com", Credentials::Anonymous).unwrap();

        let input = serde_json::json!({
            "jql": "project = DEMO"
        });

        let result = example_async_mcp_handler(&jira, "jira_search_issues", input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_entity_simulations() {
        let users = simulate_user_list_to_mcp().await;
        assert_eq!(users.len(), 2);
        assert!(users[0].name.contains("John Doe"));

        let versions = simulate_version_list_to_mcp().await;
        assert_eq!(versions.len(), 2);
        assert!(versions[0].name.contains("v1.0.0"));

        let boards = simulate_board_list_to_mcp().await;
        assert_eq!(boards.len(), 2);
        assert!(boards[0].name.contains("Kanban"));

        let sprints = simulate_sprint_list_to_mcp().await;
        assert_eq!(sprints.len(), 2);
        assert!(sprints[0].name.contains("Sprint 1"));

        let components = simulate_component_list_to_mcp().await;
        assert_eq!(components.len(), 2);
        assert!(components[0].name.contains("Frontend"));
    }

    #[tokio::test]
    async fn test_extended_mcp_handler_tools() {
        let jira = Jira::new("https://jira.example.com", Credentials::Anonymous).unwrap();

        // Test transition tools
        let input = serde_json::json!({ "issue_key": "DEMO-123" });
        let result = example_async_mcp_handler(&jira, "jira_list_issue_transitions", input).await;
        assert!(result.is_ok());

        let input = serde_json::json!({ "issue_key": "DEMO-123", "transition_id": "31" });
        let result = example_async_mcp_handler(&jira, "jira_trigger_issue_transition", input).await;
        assert!(result.is_ok());

        // Test attachment tools
        let input = serde_json::json!({ "issue_key": "DEMO-123" });
        let result = example_async_mcp_handler(&jira, "jira_list_issue_attachments", input).await;
        assert!(result.is_ok());

        let input = serde_json::json!({ "issue_key": "DEMO-123", "filename": "test.txt" });
        let result = example_async_mcp_handler(&jira, "jira_upload_issue_attachment", input).await;
        assert!(result.is_ok());

        // Test component tools
        let input = serde_json::json!({ "project_key": "DEMO", "name": "New Component" });
        let result = example_async_mcp_handler(&jira, "jira_create_project_component", input).await;
        assert!(result.is_ok());

        let input = serde_json::json!({ "component_id": "10001" });
        let result = example_async_mcp_handler(&jira, "jira_update_project_component", input).await;
        assert!(result.is_ok());
    }
}
