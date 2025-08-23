//! Async project management example
//!
//! This example demonstrates how to use the async Projects interface to manage
//! Jira projects asynchronously. Requires the "async" feature to be enabled.

#[cfg(feature = "async")]
use gouqi::{Credentials, ProjectSearchOptions, r#async::Jira};
#[cfg(feature = "async")]
use std::env;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up async Jira client
    let host =
        env::var("JIRA_HOST").unwrap_or_else(|_| "https://your-domain.atlassian.net".to_string());
    let username = env::var("JIRA_USER").unwrap_or_else(|_| "your-email@company.com".to_string());
    let token = env::var("JIRA_TOKEN").unwrap_or_else(|_| "your-api-token".to_string());

    let jira = Jira::new(host, Credentials::Basic(username, token))?;

    println!("=== Async Jira Project Management Example ===");

    // 1. List all accessible projects asynchronously
    println!("\n1. Listing all accessible projects (async):");
    match jira.projects().list().await {
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

    // 2. Get project details concurrently for multiple projects
    let project_keys = vec!["DEMO", "TEST", "PROJ"]; // Modify to match your projects
    println!("\n2. Getting multiple projects concurrently:");

    let mut tasks = Vec::new();
    for key in project_keys {
        let projects_client = jira.projects();
        let task = tokio::spawn(async move { (key, projects_client.get(key).await) });
        tasks.push(task);
    }

    for task in tasks {
        let (key, result) = task.await?;
        match result {
            Ok(project) => {
                println!(
                    "  ✓ {}: {} ({})",
                    key, project.name, project.project_type_key
                );
            }
            Err(e) => {
                println!("  ✗ {}: Error - {}", key, e);
            }
        }
    }

    // 3. Search for projects with different criteria concurrently
    println!("\n3. Concurrent project searches:");

    let projects_client = jira.projects();
    let search_options1 = ProjectSearchOptions {
        query: Some("demo".to_string()),
        max_results: Some(5),
        ..Default::default()
    };
    let search_options2 = ProjectSearchOptions {
        project_type_key: Some("software".to_string()),
        max_results: Some(5),
        ..Default::default()
    };

    let search1 = projects_client.search(&search_options1);
    let search2 = projects_client.search(&search_options2);

    // Run searches concurrently
    let (demo_results, software_results) = tokio::join!(search1, search2);

    match demo_results {
        Ok(results) => {
            println!("  Demo projects: {} found", results.total);
            for project in results.values.iter().take(3) {
                println!("    - {}: {}", project.key, project.name);
            }
        }
        Err(e) => println!("  Demo search error: {}", e),
    }

    match software_results {
        Ok(results) => {
            println!("  Software projects: {} found", results.total);
            for project in results.values.iter().take(3) {
                println!("    - {}: {}", project.key, project.name);
            }
        }
        Err(e) => println!("  Software search error: {}", e),
    }

    // 4. Get project metadata concurrently
    let demo_project = "DEMO"; // Change to an existing project
    println!(
        "\n4. Getting project metadata concurrently for '{}':",
        demo_project
    );

    let projects_client = jira.projects();
    let versions_task = projects_client.get_versions(demo_project);
    let components_task = projects_client.get_components(demo_project);
    let roles_task = projects_client.get_roles(demo_project);

    let (versions_result, components_result, roles_result) =
        tokio::join!(versions_task, components_task, roles_task);

    match versions_result {
        Ok(versions) => {
            println!(
                "  Versions ({}): {}",
                versions.len(),
                versions
                    .iter()
                    .map(|v| v.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Err(e) => println!("  Versions error: {}", e),
    }

    match components_result {
        Ok(components) => {
            println!(
                "  Components ({}): {}",
                components.len(),
                components
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Err(e) => println!("  Components error: {}", e),
    }

    match roles_result {
        Ok(roles) => {
            println!(
                "  Roles ({}): {}",
                roles.len(),
                roles.keys().cloned().collect::<Vec<_>>().join(", ")
            );
        }
        Err(e) => println!("  Roles error: {}", e),
    }

    // 5. Create a project asynchronously (uncomment and modify as needed)
    /*
    println!("\n5. Creating a new project asynchronously:");
    let new_project = CreateProject {
        key: "ASYNCTEST".to_string(),
        name: "Async Test Project".to_string(),
        project_type_key: "software".to_string(),
        description: Some("A test project created via async API".to_string()),
        lead: Some("your-username".to_string()),
        url: None,
        assignee_type: None,
        avatar_id: None,
        issue_security_scheme: None,
        permission_scheme: None,
        notification_scheme: None,
        category_id: None,
    };

    match jira.projects().create(new_project).await {
        Ok(project) => {
            println!("  ✓ Created project: {} ({})", project.name, project.key);

            // Get the newly created project details
            match jira.projects().get(&project.key).await {
                Ok(details) => {
                    println!("  ✓ Verified project creation: {}", details.name);
                }
                Err(e) => println!("  ✗ Error verifying project: {}", e),
            }
        }
        Err(e) => println!("  ✗ Error creating project: {}", e),
    }
    */

    println!("\n=== Async Project Management Example Complete ===");
    println!("\nBenefits of async operations:");
    println!("  - Concurrent API calls reduce total execution time");
    println!("  - Non-blocking operations allow better resource utilization");
    println!("  - Ideal for batch operations and data synchronization");

    Ok(())
}

// Fallback for when async feature is not enabled
#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --features async --example async_projects");
}
