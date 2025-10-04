use gouqi::{CreateIssueLinkInput, Credentials, Jira};
use serde_json::json;

#[test]
fn test_get_issue_link() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10001",
        "self": format!("{}/rest/api/latest/issueLink/10001", server.url()),
        "type": {
            "id": "10000",
            "name": "Blocks",
            "inward": "is blocked by",
            "outward": "blocks",
            "self": format!("{}/rest/api/latest/issueLinkType/10000", server.url())
        },
        "inwardIssue": {
            "id": "10004",
            "key": "TEST-2",
            "self": format!("{}/rest/api/latest/issue/TEST-2", server.url()),
            "fields": {
                "summary": "Issue that is blocked"
            }
        },
        "outwardIssue": {
            "id": "10005",
            "key": "TEST-3",
            "self": format!("{}/rest/api/latest/issue/TEST-3", server.url()),
            "fields": {
                "summary": "Issue that blocks"
            }
        }
    });

    let mock = server
        .mock("GET", "/rest/api/latest/issueLink/10001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issue_links().get("10001");

    mock.assert();
    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.id, "10001");
    assert_eq!(link.link_type.name, "Blocks");
}

#[test]
fn test_get_issue_link_not_found() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/rest/api/latest/issueLink/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorMessages": ["Issue link not found"],
                "errors": {}
            })
            .to_string(),
        )
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issue_links().get("99999");

    mock.assert();
    assert!(result.is_err());
}

#[test]
fn test_create_issue_link() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/rest/api/latest/issueLink")
        .with_status(201)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let link_input = CreateIssueLinkInput::new("Blocks", "TEST-1", "TEST-2");
    let result = jira.issue_links().create(link_input);

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_create_issue_link_with_comment() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/rest/api/latest/issueLink")
        .match_body(mockito::Matcher::JsonString(
            json!({
                "type": {
                    "name": "Relates"
                },
                "inwardIssue": {
                    "key": "TEST-1"
                },
                "outwardIssue": {
                    "key": "TEST-2"
                },
                "comment": {
                    "body": "These issues are related"
                }
            })
            .to_string(),
        ))
        .with_status(201)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let link_input = CreateIssueLinkInput::new("Relates", "TEST-1", "TEST-2")
        .with_comment("These issues are related");
    let result = jira.issue_links().create(link_input);

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_create_issue_link_validation_error() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/rest/api/latest/issueLink")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorMessages": ["The issue does not exist"],
                "errors": {}
            })
            .to_string(),
        )
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let link_input = CreateIssueLinkInput::new("Blocks", "INVALID-1", "TEST-2");
    let result = jira.issue_links().create(link_input);

    mock.assert();
    assert!(result.is_err());
}

#[test]
fn test_delete_issue_link() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/rest/api/latest/issueLink/10001")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issue_links().delete("10001");

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_delete_issue_link_not_found() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/rest/api/latest/issueLink/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorMessages": ["Issue link not found"],
                "errors": {}
            })
            .to_string(),
        )
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issue_links().delete("99999");

    mock.assert();
    assert!(result.is_err());
}

#[test]
fn test_delete_issue_link_unauthorized() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/rest/api/latest/issueLink/10001")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorMessages": ["You do not have permission to delete this link"],
                "errors": {}
            })
            .to_string(),
        )
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issue_links().delete("10001");

    mock.assert();
    assert!(result.is_err());
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_get_issue_link() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "id": "10001",
            "self": format!("{}/rest/api/latest/issueLink/10001", server.url()),
            "type": {
                "id": "10000",
                "name": "Blocks",
                "inward": "is blocked by",
                "outward": "blocks",
                "self": format!("{}/rest/api/latest/issueLinkType/10000", server.url())
            },
            "inwardIssue": {
                "id": "10004",
                "key": "TEST-2",
                "self": format!("{}/rest/api/latest/issue/TEST-2", server.url()),
                "fields": {
                    "summary": "Issue that is blocked"
                }
            }
        });

        let mock = server
            .mock("GET", "/rest/api/latest/issueLink/10001")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issue_links().get("10001").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.id, "10001");
    }

    #[tokio::test]
    async fn test_async_create_issue_link() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", "/rest/api/latest/issueLink")
            .with_status(201)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let link_input = CreateIssueLinkInput::new("Blocks", "TEST-1", "TEST-2");
        let result = jira.issue_links().create(link_input).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_delete_issue_link() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("DELETE", "/rest/api/latest/issueLink/10001")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issue_links().delete("10001").await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }
}
