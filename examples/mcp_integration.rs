//! MCP (Model Context Protocol) Integration Examples
//!
//! This example demonstrates how to use the MCP utilities provided by gouqi
//! to integrate Jira data with MCP-compatible systems.

use gouqi::mcp::{MCPError, MCPResource};
use gouqi::mcp::{error, schema, uri, validation};
use gouqi::{Credentials, Jira};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize Jira client
    let jira = Jira::new(
        "https://jira.example.com",
        Credentials::Basic("username".to_string(), "password".to_string()),
    )?;

    println!("=== MCP Resource URI Examples ===");
    demonstrate_resource_uris();

    println!("\n=== MCP Tool Schema Examples ===");
    demonstrate_tool_schemas();

    println!("\n=== MCP Validation Examples ===");
    demonstrate_validation();

    println!("\n=== MCP Resource Conversion Examples ===");
    demonstrate_resource_conversion(&jira)?;

    println!("\n=== MCP Error Handling Examples ===");
    demonstrate_error_handling();

    Ok(())
}

/// Demonstrate MCP resource URI generation and parsing
fn demonstrate_resource_uris() {
    // Generate various resource URIs
    let issue_uri = uri::issue_uri("DEMO-123");
    let project_uri = uri::project_uri("DEMO");
    let user_uri = uri::user_uri("123456789");
    let component_uri = uri::component_uri("10001");
    let version_uri = uri::version_uri("10002");

    println!("Generated URIs:");
    println!("  Issue: {}", issue_uri);
    println!("  Project: {}", project_uri);
    println!("  User: {}", user_uri);
    println!("  Component: {}", component_uri);
    println!("  Version: {}", version_uri);

    // Parse URIs back to components
    if let Ok((resource_type, resource_id)) = uri::parse_jira_uri(&issue_uri) {
        println!(
            "Parsed issue URI: type='{}', id='{}'",
            resource_type, resource_id
        );
    }

    if let Ok((resource_type, resource_id)) = uri::parse_jira_uri(&project_uri) {
        println!(
            "Parsed project URI: type='{}', id='{}'",
            resource_type, resource_id
        );
    }

    // Validate URIs
    let uris_to_validate = vec![issue_uri, project_uri, user_uri, component_uri, version_uri];
    for uri in uris_to_validate {
        match uri::validate_jira_uri(&uri) {
            Ok(()) => println!("✓ Valid URI: {}", uri),
            Err(e) => println!("✗ Invalid URI: {} (error: {:?})", uri, e),
        }
    }
}

/// Demonstrate MCP tool schema generation
fn demonstrate_tool_schemas() {
    // Generate individual tool schemas
    let search_tool = schema::issue_search_tool();
    let get_issue_tool = schema::get_issue_tool();
    let create_issue_tool = schema::create_issue_tool();
    let list_projects_tool = schema::list_projects_tool();

    println!("Individual Tool Schemas:");
    println!(
        "  Search Tool: {} - {}",
        search_tool.name, search_tool.description
    );
    println!(
        "  Get Issue Tool: {} - {}",
        get_issue_tool.name, get_issue_tool.description
    );
    println!(
        "  Create Issue Tool: {} - {}",
        create_issue_tool.name, create_issue_tool.description
    );
    println!(
        "  List Projects Tool: {} - {}",
        list_projects_tool.name, list_projects_tool.description
    );

    // Get all available tools
    let all_tools = schema::all_tools();
    println!("\nAll Available Tools ({} total):", all_tools.len());
    for tool in all_tools {
        println!("  - {}: {}", tool.name, tool.description);

        // Show schema properties for the search tool as an example
        if tool.name == "jira_search_issues" {
            if let Some(properties) = tool.input_schema.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    let property_names: Vec<&str> = props_obj.keys().map(|s| s.as_str()).collect();
                    println!("    Properties: {}", property_names.join(", "));
                }
            }
        }
    }
}

/// Demonstrate input validation functions
fn demonstrate_validation() {
    println!("Issue Key Validation:");
    let issue_keys = vec!["DEMO-123", "PROJECT-456", "demo-123", "INVALID", ""];
    for key in issue_keys {
        match validation::validate_issue_key(key) {
            Ok(()) => println!("  ✓ Valid issue key: '{}'", key),
            Err(_) => println!("  ✗ Invalid issue key: '{}'", key),
        }
    }

    println!("\nProject Key Validation:");
    let project_keys = vec!["DEMO", "PROJECT", "demo", "DEMO-PROJECT", "TOOLONGPROJECT"];
    for key in project_keys {
        match validation::validate_project_key(key) {
            Ok(()) => println!("  ✓ Valid project key: '{}'", key),
            Err(_) => println!("  ✗ Invalid project key: '{}'", key),
        }
    }

    println!("\nJQL Validation:");
    let jql_queries = vec![
        "project = DEMO",
        "assignee = currentUser()",
        "status = 'In Progress' AND project = DEMO",
        "DROP TABLE users",          // This should be invalid
        "project = DEMO -- comment", // This should be invalid
    ];
    for jql in jql_queries {
        match validation::validate_jql(jql) {
            Ok(()) => println!("  ✓ Valid JQL: '{}'", jql),
            Err(_) => println!("  ✗ Invalid JQL: '{}'", jql),
        }
    }

    println!("\nPagination Validation:");
    let pagination_params = vec![
        (Some(0), Some(50)),
        (Some(100), Some(25)),
        (Some(-1), Some(50)),  // This should be invalid
        (Some(0), Some(1001)), // This should be invalid
    ];
    for (start_at, max_results) in pagination_params {
        match validation::validate_pagination(start_at, max_results) {
            Ok(()) => println!(
                "  ✓ Valid pagination: start_at={:?}, max_results={:?}",
                start_at, max_results
            ),
            Err(_) => println!(
                "  ✗ Invalid pagination: start_at={:?}, max_results={:?}",
                start_at, max_results
            ),
        }
    }
}

/// Demonstrate converting Jira entities to MCP resources
fn demonstrate_resource_conversion(_jira: &Jira) -> Result<(), Box<dyn std::error::Error>> {
    println!("Converting Jira entities to MCP resources...");

    // In a real application, you would fetch actual data from Jira
    // For this example, we'll show the structure without making actual API calls
    println!("Note: In a real application, you would:");
    println!("1. Fetch issues using: jira.search().list(\"project = DEMO\", &Default::default())?");
    println!("2. Fetch projects using: jira.projects().iter()?");
    println!("3. Convert each entity using the ToMCPResource trait");

    // Show what the conversion would look like
    println!("\nExample MCP resource structure for an issue:");
    let example_mcp_resource = MCPResource {
        uri: uri::issue_uri("DEMO-123"),
        name: "DEMO-123: Fix login bug".to_string(),
        description: Some("Users are unable to log in with their credentials".to_string()),
        mime_type: "application/json".to_string(),
        annotations: Some({
            let mut annotations = HashMap::new();
            annotations.insert("project".to_string(), serde_json::json!("DEMO"));
            annotations.insert("status".to_string(), serde_json::json!("In Progress"));
            annotations.insert("issue_type".to_string(), serde_json::json!("Bug"));
            annotations.insert("assignee".to_string(), serde_json::json!("John Doe"));
            annotations
        }),
    };

    println!("  URI: {}", example_mcp_resource.uri);
    println!("  Name: {}", example_mcp_resource.name);
    println!(
        "  Description: {}",
        example_mcp_resource
            .description
            .as_ref()
            .unwrap_or(&"None".to_string())
    );
    println!("  MIME Type: {}", example_mcp_resource.mime_type);
    if let Some(annotations) = &example_mcp_resource.annotations {
        println!("  Annotations:");
        for (key, value) in annotations {
            println!("    {}: {}", key, value);
        }
    }

    Ok(())
}

/// Demonstrate MCP error handling and conversion
fn demonstrate_error_handling() {
    println!("Converting Jira errors to MCP format:");

    // Simulate different types of Jira errors and convert them to MCP format
    let jira_errors = vec![
        gouqi::Error::Unauthorized,
        gouqi::Error::NotFound,
        gouqi::Error::MethodNotAllowed,
    ];

    for jira_error in jira_errors {
        let mcp_error = error::to_mcp_error(&jira_error);
        println!("  Jira Error: {:?}", jira_error);
        println!(
            "    → MCP Error: code={}, message=\"{}\"",
            mcp_error.code, mcp_error.message
        );
        if let Some(data) = &mcp_error.data {
            if let Some(error_type) = data.get("type") {
                println!("      Type: {}", error_type);
            }
        }
        println!();
    }
}

/// Example of how to implement an MCP server handler using gouqi
/// This would typically be used in a larger MCP server implementation
#[allow(dead_code)]
fn example_mcp_handler(
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

            // In a real implementation, you would:
            // 1. Use the Jira client to search for issues
            // 2. Convert results to MCP resources
            // 3. Return the resources

            Ok(serde_json::json!({
                "message": "Would search for issues with JQL",
                "jql": jql
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
            // 1. Use jira.issues().get(issue_key)?
            // 2. Convert to MCP resource using ToMCPResource trait
            // 3. Return the resource

            Ok(serde_json::json!({
                "message": "Would get issue",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_mcp_handler_search() {
        let input = serde_json::json!({
            "jql": "project = DEMO"
        });

        let result = example_mcp_handler("jira_search_issues", input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_example_mcp_handler_get_issue() {
        let input = serde_json::json!({
            "issue_key": "DEMO-123"
        });

        let result = example_mcp_handler("jira_get_issue", input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_example_mcp_handler_invalid_tool() {
        let input = serde_json::json!({});

        let result = example_mcp_handler("invalid_tool", input);
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code, 400);
            assert!(error.message.contains("Unknown tool"));
        }
    }

    #[test]
    fn test_example_mcp_handler_missing_jql() {
        let input = serde_json::json!({});

        let result = example_mcp_handler("jira_search_issues", input);
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code, 400);
            assert!(error.message.contains("Missing required parameter: jql"));
        }
    }

    #[test]
    fn test_example_mcp_handler_invalid_jql() {
        let input = serde_json::json!({
            "jql": "DROP TABLE users"
        });

        let result = example_mcp_handler("jira_search_issues", input);
        assert!(result.is_err());
    }
}
