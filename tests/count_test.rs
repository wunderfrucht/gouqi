//! Tests for issue counting functionality

use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_count_with_v2_mock() {
    let mut server = mockito::Server::new();

    // Mock v2 API response with maxResults=0 to simulate exact count
    let mock_results = json!({
        "startAt": 0,
        "maxResults": 0,
        "total": 42,
        "issues": []
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*maxResults=0.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test with prefer_exact=true (should work on v2)
    let count = jira.search().count("project = TEST", true).unwrap();

    assert_eq!(count.count, 42);
    assert!(count.is_exact, "V2 API should return exact count");
}

#[test]
fn test_count_with_v2_mock_prefer_fast() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 0,
        "maxResults": 0,
        "total": 100,
        "issues": []
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*maxResults=0.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test with prefer_exact=false (v2 still returns exact)
    let count = jira.search().count("status = Open", false).unwrap();

    assert_eq!(count.count, 100);
    assert!(count.is_exact, "V2 API only has exact count");
}

#[test]
fn test_count_empty_results() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 0,
        "maxResults": 0,
        "total": 0,
        "issues": []
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let count = jira.search().count("project = NONEXISTENT", true).unwrap();

    assert_eq!(count.count, 0);
    assert!(count.is_exact);
}

#[test]
fn test_count_with_v3_mock_approximate() {
    let mut server = mockito::Server::new();

    // Mock v3 API approximate-count endpoint
    let mock_response = json!({
        "count": 1234
    });

    server
        .mock("POST", "/rest/api/3/search/approximate-count")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    // Create a Jira client configured to use V3 API
    let jira = Jira::with_search_api_version(
        server.url(),
        Credentials::Anonymous,
        gouqi::SearchApiVersion::V3,
    )
    .unwrap();

    // Test with prefer_exact=false
    let count = jira.search().count("project = TEST", false).unwrap();

    assert_eq!(count.count, 1234);
    assert!(!count.is_exact, "V3 API should return approximate count");
}

#[test]
fn test_count_with_v3_mock_prefer_exact_ignored() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "count": 5678
    });

    server
        .mock("POST", "/rest/api/3/search/approximate-count")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::with_search_api_version(
        server.url(),
        Credentials::Anonymous,
        gouqi::SearchApiVersion::V3,
    )
    .unwrap();

    // Test with prefer_exact=true (should be ignored on V3)
    let count = jira.search().count("project = TEST", true).unwrap();

    assert_eq!(count.count, 5678);
    assert!(
        !count.is_exact,
        "V3 API should return approximate count even when prefer_exact=true"
    );
}

#[test]
fn test_count_with_v3_mock_zero_results() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "count": 0
    });

    server
        .mock("POST", "/rest/api/3/search/approximate-count")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::with_search_api_version(
        server.url(),
        Credentials::Anonymous,
        gouqi::SearchApiVersion::V3,
    )
    .unwrap();

    let count = jira.search().count("project = EMPTY", false).unwrap();

    assert_eq!(count.count, 0);
    assert!(!count.is_exact);
}

#[cfg(feature = "async")]
mod async_count_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_count() {
        let mut server = mockito::Server::new_async().await;

        let mock_results = json!({
            "startAt": 0,
            "maxResults": 0,
            "total": 25,
            "issues": []
        });

        server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*maxResults=0.*".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_results.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let count = jira.search().count("project = ASYNC", true).await.unwrap();

        assert_eq!(count.count, 25);
        assert!(count.is_exact);
    }

    #[tokio::test]
    async fn test_async_count_v3_approximate() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "count": 999
        });

        server
            .mock("POST", "/rest/api/3/search/approximate-count")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::with_search_api_version(
            server.url(),
            Credentials::Anonymous,
            gouqi::SearchApiVersion::V3,
        )
        .unwrap();

        let count = jira
            .search()
            .count("project = ASYNCV3", false)
            .await
            .unwrap();

        assert_eq!(count.count, 999);
        assert!(!count.is_exact, "V3 async should return approximate");
    }

    #[tokio::test]
    async fn test_async_count_v3_prefer_exact_ignored() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "count": 777
        });

        server
            .mock("POST", "/rest/api/3/search/approximate-count")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::with_search_api_version(
            server.url(),
            Credentials::Anonymous,
            gouqi::SearchApiVersion::V3,
        )
        .unwrap();

        // prefer_exact=true should be ignored on V3
        let count = jira
            .search()
            .count("project = ASYNCV3", true)
            .await
            .unwrap();

        assert_eq!(count.count, 777);
        assert!(
            !count.is_exact,
            "V3 async should ignore prefer_exact and return approximate"
        );
    }

    #[tokio::test]
    async fn test_async_count_v3_zero_results() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "count": 0
        });

        server
            .mock("POST", "/rest/api/3/search/approximate-count")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::with_search_api_version(
            server.url(),
            Credentials::Anonymous,
            gouqi::SearchApiVersion::V3,
        )
        .unwrap();

        let count = jira.search().count("project = EMPTY", false).await.unwrap();

        assert_eq!(count.count, 0);
        assert!(!count.is_exact);
    }
}

// Real integration tests with actual JIRA Cloud instance
#[test]
fn test_real_count_with_scrum_project() {
    use std::env;

    // Skip if no credentials provided
    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real count test");
            return;
        }
    };

    println!("🧪 Testing count() with real JIRA Cloud instance...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Verify we're using V3 API
    assert_eq!(jira.get_search_api_version(), gouqi::SearchApiVersion::V3);
    println!("✅ Confirmed V3 API detected");

    // Test counting issues in SCRUM project
    println!("🔍 Counting issues in SCRUM project...");
    let count = jira
        .search()
        .count("project = SCRUM", false)
        .expect("Failed to count issues");

    println!(
        "📊 Count result: {} issues ({})",
        count.count,
        if count.is_exact {
            "exact"
        } else {
            "approximate"
        }
    );

    // V3 API should return approximate count for now
    assert!(!count.is_exact, "V3 API should return approximate count");

    // Verify we got a reasonable count (SCRUM project should have some issues)
    assert!(
        count.count > 0,
        "SCRUM project should have at least some issues"
    );

    println!("✅ Count test passed!");
}

#[test]
fn test_real_count_prefer_exact() {
    use std::env;

    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping real count test");
            return;
        }
    };

    println!("🧪 Testing count() with prefer_exact=true...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Request exact count
    let exact_count = jira
        .search()
        .count("project = SCRUM", true)
        .expect("Failed to count issues");

    println!(
        "📊 Exact count request result: {} issues ({})",
        exact_count.count,
        if exact_count.is_exact {
            "exact"
        } else {
            "approximate - exact unavailable"
        }
    );

    // Even with prefer_exact=true, v3 currently returns approximate
    assert!(
        !exact_count.is_exact,
        "V3 API doesn't support exact count yet"
    );

    println!("✅ Exact count preference test passed (returned approximate as expected)!");
}

#[test]
fn test_real_count_with_jql_filter() {
    use std::env;

    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping JQL filter count test");
            return;
        }
    };

    println!("🧪 Testing count() with JQL filter...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create Jira client");

    // Count open issues in SCRUM project
    let open_count = jira
        .search()
        .count("project = SCRUM AND status != Done", false)
        .expect("Failed to count open issues");

    println!(
        "📊 Open issues in SCRUM: {} ({})",
        open_count.count,
        if open_count.is_exact {
            "exact"
        } else {
            "approximate"
        }
    );

    // Count all issues
    let total_count = jira
        .search()
        .count("project = SCRUM", false)
        .expect("Failed to count total issues");

    println!("📊 Total issues in SCRUM: {}", total_count.count);

    // Open issues should be <= total issues
    assert!(
        open_count.count <= total_count.count,
        "Open issues ({}) should not exceed total issues ({})",
        open_count.count,
        total_count.count
    );

    println!("✅ JQL filter count test passed!");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_real_async_count() {
    use std::env;

    let token = match env::var("JIRA_PASSWORD") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            eprintln!("⚠️  JIRA_PASSWORD not set, skipping async count test");
            return;
        }
    };

    println!("🧪 Testing async count()...");

    let username = env::var("JIRA_USERNAME").unwrap_or_else(|_| "rbeier57@gmail.com".to_string());
    let jira = gouqi::r#async::Jira::new(
        "https://gouji.atlassian.net",
        Credentials::Basic(username, token),
    )
    .expect("Failed to create async Jira client");

    let count = jira
        .search()
        .count("project = SCRUM", false)
        .await
        .expect("Failed to count issues");

    println!(
        "📊 Async count result: {} issues ({})",
        count.count,
        if count.is_exact {
            "exact"
        } else {
            "approximate"
        }
    );

    assert!(count.count > 0, "SCRUM project should have issues");
    assert!(!count.is_exact, "V3 should return approximate");

    println!("✅ Async count test passed!");
}
