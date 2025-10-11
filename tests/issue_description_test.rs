//! Tests for Issue::description() with both string and ADF formats

use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_description_legacy_string_format() {
    let mut server = mockito::Server::new();

    // Legacy v2 API format - description as plain string
    let response_data = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
        "fields": {
            "summary": "Test issue with string description",
            "description": "This is a plain text description from JIRA v2 API"
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-1").unwrap();

    assert_eq!(issue.key, "TEST-1");
    assert_eq!(
        issue.description(),
        Some("This is a plain text description from JIRA v2 API".to_string())
    );
}

#[test]
fn test_description_adf_format_simple() {
    let mut server = mockito::Server::new();

    // v3 API format - description as ADF (Atlassian Document Format)
    let response_data = json!({
        "id": "10002",
        "key": "TEST-2",
        "self": format!("{}/rest/api/latest/issue/TEST-2", server.url()),
        "fields": {
            "summary": "Test issue with ADF description",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "This is a description in ADF format."
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-2").unwrap();

    assert_eq!(issue.key, "TEST-2");
    assert_eq!(
        issue.description(),
        Some("This is a description in ADF format.".to_string())
    );
}

#[test]
fn test_description_adf_format_multiline() {
    let mut server = mockito::Server::new();

    // v3 API format with multiple paragraphs
    let response_data = json!({
        "id": "10003",
        "key": "TEST-3",
        "self": format!("{}/rest/api/latest/issue/TEST-3", server.url()),
        "fields": {
            "summary": "Test issue with multiline ADF description",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "First paragraph of the description."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Second paragraph with more details."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Third paragraph with even more information."
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-3")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-3").unwrap();

    assert_eq!(issue.key, "TEST-3");
    let description = issue.description().unwrap();
    assert!(description.contains("First paragraph of the description."));
    assert!(description.contains("Second paragraph with more details."));
    assert!(description.contains("Third paragraph with even more information."));
    // Paragraphs should be separated by newlines
    assert_eq!(
        description,
        "First paragraph of the description.\nSecond paragraph with more details.\nThird paragraph with even more information."
    );
}

#[test]
fn test_description_adf_format_empty() {
    let mut server = mockito::Server::new();

    // v3 API format with empty content
    let response_data = json!({
        "id": "10004",
        "key": "TEST-4",
        "self": format!("{}/rest/api/latest/issue/TEST-4", server.url()),
        "fields": {
            "summary": "Test issue with empty ADF description",
            "description": {
                "version": 1,
                "type": "doc",
                "content": []
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-4")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-4").unwrap();

    assert_eq!(issue.key, "TEST-4");
    assert_eq!(issue.description(), None);
}

#[test]
fn test_description_missing() {
    let mut server = mockito::Server::new();

    // Issue with no description field at all
    let response_data = json!({
        "id": "10005",
        "key": "TEST-5",
        "self": format!("{}/rest/api/latest/issue/TEST-5", server.url()),
        "fields": {
            "summary": "Test issue without description"
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-5")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-5").unwrap();

    assert_eq!(issue.key, "TEST-5");
    assert_eq!(issue.description(), None);
}

#[test]
fn test_description_adf_with_inline_formatting() {
    let mut server = mockito::Server::new();

    // v3 API format with formatted text (bold, italic, etc.)
    let response_data = json!({
        "id": "10006",
        "key": "TEST-6",
        "self": format!("{}/rest/api/latest/issue/TEST-6", server.url()),
        "fields": {
            "summary": "Test issue with formatted ADF description",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "This text has "
                            },
                            {
                                "type": "text",
                                "text": "bold formatting",
                                "marks": [
                                    {
                                        "type": "strong"
                                    }
                                ]
                            },
                            {
                                "type": "text",
                                "text": " and "
                            },
                            {
                                "type": "text",
                                "text": "italic formatting",
                                "marks": [
                                    {
                                        "type": "em"
                                    }
                                ]
                            },
                            {
                                "type": "text",
                                "text": " mixed together."
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-6")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-6").unwrap();

    assert_eq!(issue.key, "TEST-6");
    // The formatting marks are ignored, just the plain text is extracted
    assert_eq!(
        issue.description(),
        Some("This text has bold formatting and italic formatting mixed together.".to_string())
    );
}

#[test]
fn test_adf_to_plain_text_roundtrip() {
    use gouqi::AdfDocument;

    // Test that we can convert text to ADF and back
    let original_text = "Line 1\nLine 2\nLine 3";
    let adf = AdfDocument::from_text(original_text);
    let extracted_text = adf.to_plain_text();

    assert_eq!(extracted_text, original_text);
}

#[test]
fn test_adf_to_plain_text_empty() {
    use gouqi::AdfDocument;

    let adf = AdfDocument::from_text("");
    let extracted_text = adf.to_plain_text();

    assert_eq!(extracted_text, "");
}
