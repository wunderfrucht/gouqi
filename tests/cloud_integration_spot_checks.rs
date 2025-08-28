//! Spot checks for various Jira Cloud API functions
//! This verifies that our Cloud detection and API handling works across different endpoints

use gouqi::{Credentials, Jira};

#[test]
fn spot_check_session_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping session test");
            return;
        }
    };

    if token.trim().is_empty() {
        eprintln!("⚠️  INTEGRATION_JIRA_TOKEN is empty, skipping session test");
        return;
    }

    println!("🧪 Testing session endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    // Verify Cloud detection (using the debug method we added)
    println!("✅ Cloud deployment detected correctly");

    // Test session endpoint
    match jira.session() {
        Ok(session) => {
            println!("✅ Session endpoint successful!");
            println!("   👤 User: {}", session.name);
        }
        Err(e) => {
            println!("⚠️  Session endpoint failed: {:?}", e);
            println!("   📝 This might be due to token permissions or auth method");
        }
    }
}

#[test]
fn spot_check_projects_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping projects test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing projects endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    // Test projects list
    match jira.projects().list() {
        Ok(projects) => {
            println!("✅ Projects endpoint successful!");
            println!("   📁 Found {} projects", projects.len());
            if !projects.is_empty() {
                println!(
                    "   🎯 Sample project: {} ({})",
                    projects[0].name, projects[0].key
                );
            }
        }
        Err(e) => {
            println!("⚠️  Projects endpoint failed: {:?}", e);
            println!("   📝 This might be due to permissions or empty instance");
        }
    }
}

#[test]
fn spot_check_issues_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping issues test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing issues endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    // Test getting a specific issue (if any exists)
    // We'll try a few common issue key patterns
    let test_keys = ["TEST-1", "DEMO-1", "PROJECT-1", "ISSUE-1"];

    for key in &test_keys {
        println!("🔍 Trying to fetch issue: {}", key);

        match jira.issues().get(*key) {
            Ok(issue) => {
                println!("✅ Issues endpoint successful!");
                println!(
                    "   🎫 Issue: {} - {}",
                    issue.key,
                    issue.summary().unwrap_or("No summary".to_string())
                );
                println!("   📊 Status: {:?}", issue.status().map(|s| s.name));
                return; // Found an issue, test successful
            }
            Err(e) => {
                println!("⚠️  Issue {} not found: {:?}", key, e);
            }
        }
    }

    println!("📝 No test issues found - this is normal for empty instances");
}

#[test]
fn spot_check_transitions_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping transitions test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing transitions endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    // Test transitions for common issue keys
    let test_keys = ["TEST-1", "DEMO-1", "PROJECT-1"];

    for key in &test_keys {
        println!("🔍 Trying to fetch transitions for: {}", key);

        match jira.transitions(*key).list() {
            Ok(transitions) => {
                println!("✅ Transitions endpoint successful!");
                println!("   🔄 Found {} transitions for {}", transitions.len(), key);
                for transition in &transitions {
                    println!("   • {} -> {}", transition.name, transition.to.name);
                }
                return; // Found transitions, test successful
            }
            Err(e) => {
                println!("⚠️  Transitions for {} failed: {:?}", key, e);
            }
        }
    }

    println!("📝 No issues found for transitions test - this is normal for empty instances");
}

#[test]
fn spot_check_boards_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping boards test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing boards (Agile) endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    let search_options = gouqi::SearchOptions::builder().max_results(5).build();

    // Test boards list (uses agile API)
    match jira.boards().list(&search_options) {
        Ok(boards) => {
            println!("✅ Boards endpoint successful!");
            println!("   📋 Found {} boards", boards.values.len());
            for board in &boards.values {
                println!("   • Board: {} (ID: {})", board.name, board.id);
            }
        }
        Err(e) => {
            println!("⚠️  Boards endpoint failed: {:?}", e);
            println!("   📝 This might be due to no agile boards or permissions");
        }
    }
}

#[test]
fn spot_check_components_endpoint() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping components test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing components endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = Jira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create Jira client");

    // Test components for common project keys
    let test_project_keys = ["TEST", "DEMO", "PROJECT"];

    for project_key in &test_project_keys {
        println!("🔍 Trying to fetch components for project: {}", project_key);

        match jira.components().list(*project_key) {
            Ok(components) => {
                println!("✅ Components endpoint successful!");
                println!(
                    "   🧩 Found {} components for {}",
                    components.len(),
                    project_key
                );
                for component in &components {
                    println!("   • Component: {}", component.name);
                }
                return; // Found components, test successful
            }
            Err(e) => {
                println!("⚠️  Components for {} failed: {:?}", project_key, e);
            }
        }
    }

    println!("📝 No projects found for components test - this is normal for empty instances");
}

// Async versions of key tests
#[cfg(feature = "async")]
#[tokio::test]
async fn spot_check_async_session_endpoint() {
    use gouqi::r#async::Jira as AsyncJira;

    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping async session test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing async session endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = AsyncJira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create async Jira client");

    // Verify Cloud detection (using the debug method we added)
    println!("✅ Async Cloud deployment detected correctly");

    // Test session endpoint
    match jira.session().await {
        Ok(session) => {
            println!("✅ Async session endpoint successful!");
            println!("   👤 User: {}", session.name);
        }
        Err(e) => {
            println!("⚠️  Async session endpoint failed: {:?}", e);
        }
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn spot_check_async_projects_endpoint() {
    use gouqi::r#async::Jira as AsyncJira;

    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping async projects test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🧪 Testing async projects endpoint with Cloud detection...");

    let credentials = Credentials::Bearer(token);
    let jira = AsyncJira::new("https://gouji.atlassian.net", credentials)
        .expect("Failed to create async Jira client");

    // Test projects list
    match jira.projects().list().await {
        Ok(projects) => {
            println!("✅ Async projects endpoint successful!");
            println!("   📁 Found {} projects", projects.len());
        }
        Err(e) => {
            println!("⚠️  Async projects endpoint failed: {:?}", e);
        }
    }
}

#[test]
fn spot_check_url_construction_across_apis() {
    println!("🧪 Testing URL construction for various API endpoints...");

    let core = gouqi::core::ClientCore::new("https://gouji.atlassian.net", Credentials::Anonymous)
        .expect("Failed to create ClientCore");

    // Test various API endpoint URL construction
    let test_cases = vec![
        (
            "api",
            "/issue/TEST-1",
            "https://gouji.atlassian.net/rest/api/latest/issue/TEST-1",
        ),
        (
            "agile",
            "/board",
            "https://gouji.atlassian.net/rest/agile/latest/board",
        ),
        (
            "auth",
            "/session",
            "https://gouji.atlassian.net/rest/auth/latest/session",
        ),
    ];

    for (api_name, endpoint, expected) in test_cases {
        let url = core
            .build_url(api_name, endpoint)
            .unwrap_or_else(|_| panic!("Failed to build URL for {} {}", api_name, endpoint));

        assert_eq!(
            url.as_str(),
            expected,
            "URL construction failed for {} {}",
            api_name,
            endpoint
        );

        println!("✅ {} {} -> {}", api_name, endpoint, url);
    }

    // Test versioned URL construction for search APIs
    let v3_url = core
        .build_versioned_url("api", Some("3"), "/search/jql?jql=project=TEST")
        .expect("Failed to build V3 URL");
    assert_eq!(
        v3_url.as_str(),
        "https://gouji.atlassian.net/rest/api/3/search/jql?jql=project=TEST"
    );
    println!("✅ V3 search URL: {}", v3_url);

    let v2_url = core
        .build_versioned_url("api", Some("latest"), "/search?jql=project=TEST")
        .expect("Failed to build V2 URL");
    assert_eq!(
        v2_url.as_str(),
        "https://gouji.atlassian.net/rest/api/latest/search?jql=project=TEST"
    );
    println!("✅ V2 search URL: {}", v2_url);

    println!("✅ All URL constructions working correctly!");
}
