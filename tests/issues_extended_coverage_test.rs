use gouqi::issues::{BulkIssueUpdate, BulkUpdateRequest, EditCustomIssue};
use gouqi::{Credentials, Jira};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
struct CustomFields {
    summary: String,
    #[serde(rename = "customfield_10001")]
    custom_field: Option<String>,
}

#[test]
fn test_get_custom_issue() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "fields": {
            "summary": "Custom Issue",
            "customfield_10001": "Custom Value"
        }
    });

    let mock = server
        .mock("GET", "/rest/api/latest/issue/TEST-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira
        .issues()
        .get_custom_issue::<&str, CustomFields>("TEST-1");

    mock.assert();
    assert!(result.is_ok());
    let custom_issue = result.unwrap();
    assert_eq!(custom_issue.fields.summary, "Custom Issue");
    assert_eq!(
        custom_issue.fields.custom_field,
        Some("Custom Value".to_string())
    );
}

#[test]
fn test_create_from_custom_issue() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/10001", server.url())
    });

    let mock = server
        .mock("POST", "/rest/api/latest/issue")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let custom_fields = CustomFields {
        summary: "New Custom Issue".to_string(),
        custom_field: Some("Custom Value".to_string()),
    };

    let custom_issue = gouqi::issues::CreateCustomIssue {
        fields: custom_fields,
    };

    let result = jira.issues().create_from_custom_issue(custom_issue);

    mock.assert();
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.key, "TEST-1");
}

#[test]
fn test_update_custom_issue() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("PUT", "/rest/api/latest/issue/TEST-1")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let custom_fields = CustomFields {
        summary: "Updated Custom Issue".to_string(),
        custom_field: Some("Updated Value".to_string()),
    };

    let custom_issue = EditCustomIssue {
        fields: custom_fields,
    };

    let result = jira.issues().update_custom_issue("TEST-1", custom_issue);

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_issues_iterator_empty() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 0,
        "issues": []
    });

    let mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/agile/latest/board/1/issue.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = gouqi::Board {
        id: 1,
        name: "Test Board".to_string(),
        type_name: "scrum".to_string(),
        self_link: format!("{}/rest/agile/latest/board/1", server.url()),
        location: None,
    };

    let options = Default::default();
    let iter = jira.issues().iter(&board, &options).unwrap();
    let issues: Vec<_> = iter.collect();

    mock.assert();
    assert_eq!(issues.len(), 0);
}

// Note: bulk_create tests are skipped because they require complex nested
// type construction with CreateIssue which is better tested in integration
// tests with real data. The API signature is covered by type checking.

#[test]
fn test_bulk_update_issues_success() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "issues": [
            {
                "id": "10001",
                "key": "TEST-1",
                "self": format!("{}/rest/api/latest/issue/10001", server.url()),
                "fields": {
                    "summary": "Updated Issue 1"
                }
            }
        ],
        "errors": []
    });

    let mock = server
        .mock("PUT", "/rest/api/latest/issue/bulk")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let mut fields = BTreeMap::new();
    fields.insert("summary".to_string(), json!("Updated Issue 1"));

    let update = BulkIssueUpdate {
        key: "TEST-1".to_string(),
        fields,
    };

    let request = BulkUpdateRequest {
        issue_updates: vec![update],
    };

    let result = jira.issues().bulk_update(request);

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_get_relationship_graph_single_issue_no_links() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
        "fields": {
            "summary": "Single Issue",
            "issuelinks": []
        }
    });

    let mock = server
        .mock("GET", "/rest/api/latest/issue/TEST-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get_relationship_graph("TEST-1", 0, None);

    mock.assert();
    assert!(result.is_ok());
    let graph = result.unwrap();
    assert_eq!(graph.metadata.root_issue, Some("TEST-1".to_string()));
    assert_eq!(graph.metadata.max_depth, 0);
}

#[test]
fn test_get_bulk_relationships_multiple_issues() {
    let mut server = mockito::Server::new();

    let response1 = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
        "fields": {
            "summary": "Issue 1",
            "issuelinks": []
        }
    });

    let response2 = json!({
        "id": "10002",
        "key": "TEST-2",
        "self": format!("{}/rest/api/latest/issue/TEST-2", server.url()),
        "fields": {
            "summary": "Issue 2",
            "issuelinks": []
        }
    });

    let mock1 = server
        .mock("GET", "/rest/api/latest/issue/TEST-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response1.to_string())
        .create();

    let mock2 = server
        .mock("GET", "/rest/api/latest/issue/TEST-2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response2.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue_keys = vec!["TEST-1".to_string(), "TEST-2".to_string()];
    let result = jira.issues().get_bulk_relationships(&issue_keys, None);

    mock1.assert();
    mock2.assert();
    assert!(result.is_ok());
    let graph = result.unwrap();
    assert_eq!(graph.metadata.max_depth, 0);
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_get_issue() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "id": "10001",
            "key": "TEST-1",
            "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
            "fields": {
                "summary": "Test Issue"
            }
        });

        let mock = server
            .mock("GET", "/rest/api/latest/issue/TEST-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().get("TEST-1").await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    // Note: async bulk_create test skipped - same reason as sync version

    #[tokio::test]
    async fn test_async_get_relationship_graph() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "id": "10001",
            "key": "TEST-1",
            "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
            "fields": {
                "summary": "Single Issue",
                "issuelinks": []
            }
        });

        let mock = server
            .mock("GET", "/rest/api/latest/issue/TEST-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .issues()
            .get_relationship_graph("TEST-1", 0, None)
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_get_bulk_relationships() {
        let mut server = mockito::Server::new_async().await;

        let response = json!({
            "id": "10001",
            "key": "TEST-1",
            "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
            "fields": {
                "summary": "Issue 1",
                "issuelinks": []
            }
        });

        let mock = server
            .mock("GET", "/rest/api/latest/issue/TEST-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let issue_keys = vec!["TEST-1".to_string()];
        let result = jira
            .issues()
            .get_bulk_relationships(&issue_keys, None)
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }
}
