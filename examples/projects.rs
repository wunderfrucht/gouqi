//! Project management example
//!
//! This example demonstrates how to use the Projects interface to manage
//! Jira projects, including listing, creating, updating, and working with
//! project components and versions.

use gouqi::{Credentials, Jira, ProjectSearchOptions};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up Jira client
    let host =
        env::var("JIRA_HOST").unwrap_or_else(|_| "https://your-domain.atlassian.net".to_string());
    let username = env::var("JIRA_USER").unwrap_or_else(|_| "your-email@company.com".to_string());
    let token = env::var("JIRA_TOKEN").unwrap_or_else(|_| "your-api-token".to_string());

    let jira = Jira::new(host, Credentials::Basic(username, token))?;

    println!("=== Jira Project Management Example ===");

    // 1. List all accessible projects
    println!("\n1. Listing all accessible projects:");
    match jira.projects().list() {
        Ok(projects) => {
            println!("Found {} projects:", projects.len());
            for project in &projects {
                println!("  - {} ({}): {}", project.key, project.id, project.name);
                if let Some(ref description) = project.description {
                    println!("    Description: {}", description);
                }
            }
        }
        Err(e) => println!("Error listing projects: {}", e),
    }

    // 2. Get a specific project (modify this key to match your environment)
    let project_key = "DEMO"; // Change this to an existing project key
    println!("\n2. Getting project details for '{}':", project_key);
    match jira.projects().get(project_key) {
        Ok(project) => {
            println!("Project: {} ({})", project.name, project.key);
            println!("  ID: {}", project.id);
            println!("  Type: {}", project.project_type_key);

            if let Some(ref description) = project.description {
                println!("  Description: {}", description);
            }

            if let Some(ref lead) = project.lead {
                println!("  Lead: {}", lead.display_name);
            }
        }
        Err(e) => println!("Error getting project: {}", e),
    }

    // 3. Search for projects
    println!("\n3. Searching for projects with query 'demo':");
    let search_options = ProjectSearchOptions {
        query: Some("demo".to_string()),
        start_at: Some(0),
        max_results: Some(10),
        order_by: Some("key".to_string()),
        category_id: None,
        project_type_key: Some("software".to_string()),
    };

    match jira.projects().search(&search_options) {
        Ok(results) => {
            println!("Search found {} total projects:", results.total);
            for project in results.values {
                println!("  - {} ({}): {}", project.key, project.id, project.name);
            }
        }
        Err(e) => println!("Error searching projects: {}", e),
    }

    // 4. Get project versions
    println!("\n4. Getting versions for project '{}':", project_key);
    match jira.projects().get_versions(project_key) {
        Ok(versions) => {
            println!("Found {} versions:", versions.len());
            for version in versions {
                println!(
                    "  - {} ({}): Released={}, Archived={}",
                    version.name, version.id, version.released, version.archived
                );
            }
        }
        Err(e) => println!("Error getting project versions: {}", e),
    }

    // 5. Get project components
    println!("\n5. Getting components for project '{}':", project_key);
    match jira.projects().get_components(project_key) {
        Ok(components) => {
            println!("Found {} components:", components.len());
            for component in components {
                println!("  - {} ({})", component.name, component.id);
                if let Some(ref description) = component.description {
                    println!("    Description: {}", description);
                }
            }
        }
        Err(e) => println!("Error getting project components: {}", e),
    }

    // 6. Get project roles
    println!("\n6. Getting roles for project '{}':", project_key);
    match jira.projects().get_roles(project_key) {
        Ok(roles) => {
            println!("Found {} roles:", roles.len());
            for (role_name, role_url) in roles {
                println!("  - {}: {}", role_name, role_url);
            }
        }
        Err(e) => println!("Error getting project roles: {}", e),
    }

    // 7. Create a new project (uncomment and modify as needed)
    /*
    println!("\n7. Creating a new project:");
    let new_project = CreateProject {
        key: "TEST".to_string(),
        name: "Test Project".to_string(),
        project_type_key: "software".to_string(),
        description: Some("A test project created via API".to_string()),
        lead: Some("your-username".to_string()), // Optional: set project lead
        url: None,
        assignee_type: None,
        avatar_id: None,
        issue_security_scheme: None,
        permission_scheme: None,
        notification_scheme: None,
        category_id: None,
    };

    match jira.projects().create(new_project) {
        Ok(project) => {
            println!("Created project: {} ({})", project.name, project.key);
        }
        Err(e) => println!("Error creating project: {}", e),
    }

    // 8. Update a project (uncomment and modify as needed)
    println!("\n8. Updating project description:");
    let update = UpdateProject {
        key: None,
        name: None,
        description: Some("Updated description via API".to_string()),
        lead: None,
        url: None,
        assignee_type: None,
        avatar_id: None,
        category_id: None,
    };

    match jira.projects().update("TEST", update) {
        Ok(project) => {
            println!("Updated project: {}", project.name);
        }
        Err(e) => println!("Error updating project: {}", e),
    }
    */

    println!("\n=== Project Management Example Complete ===");
    println!("\nNote: To run create/update operations, uncomment the relevant sections");
    println!("and modify the project keys to match your Jira environment.");

    Ok(())
}
