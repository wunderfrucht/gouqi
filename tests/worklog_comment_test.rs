//! Comprehensive tests for Worklog.comment() with both v2 string and v3 ADF formats
//! Tests the fix for Worklog comment handling in JIRA v3 API

use gouqi::{AdfDocument, Worklog};
use serde_json::json;

#[test]
fn test_worklog_comment_v2_string_format() {
    // Test v2 API format where comment is a plain string
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10000",
        "id": "10000",
        "comment": "Fixed the bug in production",
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(worklog.id, "10000");
    assert_eq!(
        worklog.comment(),
        Some("Fixed the bug in production".to_string())
    );
}

#[test]
fn test_worklog_comment_adf_format_simple() {
    // Test v3 API format where comment is ADF with single paragraph
    let adf = AdfDocument::from_text("Worked on feature implementation");

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10001",
        "id": "10001",
        "comment": adf,
        "timeSpent": "3h",
        "timeSpentSeconds": 10800,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(worklog.id, "10001");
    assert_eq!(
        worklog.comment(),
        Some("Worked on feature implementation".to_string())
    );
}

#[test]
fn test_worklog_comment_adf_format_multiline() {
    // Test v3 ADF with multiple paragraphs
    let text = "Completed the following tasks:\n- Fixed bug in login\n- Updated documentation";
    let adf = AdfDocument::from_text(text);

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10002",
        "id": "10002",
        "comment": adf,
        "timeSpent": "4h",
        "timeSpentSeconds": 14400,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    let comment = worklog.comment().unwrap();
    assert_eq!(comment, text);

    // Verify it has multiple lines
    assert_eq!(comment.lines().count(), 3);
}

#[test]
fn test_worklog_comment_adf_format_empty() {
    // Test v3 ADF with empty content
    let adf = AdfDocument::from_text("");

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10003",
        "id": "10003",
        "comment": adf,
        "timeSpent": "1h",
        "timeSpentSeconds": 3600,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    // Empty ADF should return None (no meaningful content)
    assert!(worklog.comment().is_none());
}

#[test]
fn test_worklog_comment_no_comment_field() {
    // Test worklog without comment field (optional field)
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10004",
        "id": "10004",
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(worklog.id, "10004");
    assert!(worklog.comment().is_none());
}

#[test]
fn test_worklog_comment_adf_with_inline_formatting() {
    // Test ADF with bold, italic marks (should be ignored in plain text extraction)
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10005",
        "id": "10005",
        "comment": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Worked on "
                        },
                        {
                            "type": "text",
                            "text": "critical bug",
                            "marks": [{"type": "strong"}]
                        },
                        {
                            "type": "text",
                            "text": " fix"
                        }
                    ]
                }
            ]
        },
        "timeSpent": "2h 30m",
        "timeSpentSeconds": 9000,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    // Should extract plain text without formatting marks
    assert_eq!(
        worklog.comment(),
        Some("Worked on critical bug fix".to_string())
    );
}

#[test]
fn test_worklog_comment_adf_with_nested_content() {
    // Test ADF with nested structures
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10006",
        "id": "10006",
        "comment": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Fixed performance issues"
                        }
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Optimized database queries"
                        }
                    ]
                }
            ]
        },
        "timeSpent": "5h",
        "timeSpentSeconds": 18000,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    let comment = worklog.comment().unwrap();
    assert_eq!(
        comment,
        "Fixed performance issues\nOptimized database queries"
    );
}

#[test]
fn test_worklog_comment_special_characters() {
    // Test v2 format with special characters
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10007",
        "id": "10007",
        "comment": "Fixed issue with <>&\"' characters",
        "timeSpent": "1h 15m",
        "timeSpentSeconds": 4500,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(
        worklog.comment(),
        Some("Fixed issue with <>&\"' characters".to_string())
    );
}

#[test]
fn test_worklog_comment_unicode() {
    // Test v3 ADF with Unicode characters
    let text = "Travail effectué: 修正 バグ 🎉";
    let adf = AdfDocument::from_text(text);

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10008",
        "id": "10008",
        "comment": adf,
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(worklog.comment(), Some(text.to_string()));
}

#[test]
fn test_worklog_comment_very_long_text() {
    // Test v3 ADF with very long text (performance test)
    let long_text = "Work description: ".to_string() + &"A".repeat(10000);
    let adf = AdfDocument::from_text(&long_text);

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10009",
        "id": "10009",
        "comment": adf,
        "timeSpent": "8h",
        "timeSpentSeconds": 28800,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    let comment = worklog.comment().unwrap();
    assert_eq!(comment.len(), long_text.len());
    assert_eq!(comment, long_text);
}

#[test]
fn test_worklog_comment_many_paragraphs() {
    // Test v3 ADF with many paragraphs
    let mut paragraphs = Vec::new();
    for i in 0..20 {
        paragraphs.push(format!("Task {}: completed", i + 1));
    }
    let text = paragraphs.join("\n");
    let adf = AdfDocument::from_text(&text);

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10010",
        "id": "10010",
        "comment": adf,
        "timeSpent": "6h",
        "timeSpentSeconds": 21600,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    let comment = worklog.comment().unwrap();
    assert_eq!(comment, text);
    assert_eq!(comment.lines().count(), 20);
}

#[test]
fn test_worklog_comment_raw_field_access() {
    // Test direct access to comment_raw field for advanced use cases
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10011",
        "id": "10011",
        "comment": "Direct field test",
        "timeSpent": "1h",
        "timeSpentSeconds": 3600,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    // Verify comment_raw contains the raw value
    assert!(worklog.comment_raw.is_some());
    assert!(worklog.comment_raw.as_ref().unwrap().is_string());
    assert_eq!(
        worklog.comment_raw.as_ref().unwrap().as_str(),
        Some("Direct field test")
    );

    // And that comment() extracts it correctly
    assert_eq!(worklog.comment(), Some("Direct field test".to_string()));
}

#[test]
fn test_worklog_comment_adf_round_trip() {
    // Test that ADF conversion is consistent
    let original_text = "Work completed:\nTask 1\nTask 2\nTask 3";
    let adf = AdfDocument::from_text(original_text);

    // Convert to JSON and back
    let adf_json = serde_json::to_value(&adf).unwrap();

    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/3/issue/TEST-1/worklog/10012",
        "id": "10012",
        "comment": adf_json,
        "timeSpent": "4h",
        "timeSpentSeconds": 14400,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    // Should extract the same text
    assert_eq!(worklog.comment(), Some(original_text.to_string()));
}

#[test]
fn test_worklog_comment_whitespace_handling() {
    // Test v2 format with various whitespace
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10013",
        "id": "10013",
        "comment": "  Spaces at start and end  \n\nMultiple blank lines\n",
        "timeSpent": "1h 30m",
        "timeSpentSeconds": 5400,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    // Should preserve whitespace as-is in v2
    assert_eq!(
        worklog.comment(),
        Some("  Spaces at start and end  \n\nMultiple blank lines\n".to_string())
    );
}

#[test]
fn test_worklog_with_all_optional_fields() {
    // Test Worklog with comprehensive field set including comment
    let worklog_json = json!({
        "self": "http://jira.example.com/rest/api/2/issue/TEST-1/worklog/10014",
        "id": "10014",
        "author": {
            "self": "http://jira.example.com/rest/api/2/user?username=john",
            "name": "john",
            "displayName": "John Doe",
            "active": true
        },
        "updateAuthor": {
            "self": "http://jira.example.com/rest/api/2/user?username=jane",
            "name": "jane",
            "displayName": "Jane Smith",
            "active": true
        },
        "comment": "Comprehensive test",
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T11:00:00.000+0000",
        "started": "2024-01-01T09:00:00.000+0000",
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    let worklog: Worklog = serde_json::from_str(&worklog_json.to_string()).unwrap();

    assert_eq!(worklog.id, "10014");
    assert_eq!(worklog.comment(), Some("Comprehensive test".to_string()));
    assert!(worklog.author.is_some());
    assert!(worklog.update_author.is_some());
    assert!(worklog.created.is_some());
    assert!(worklog.updated.is_some());
    assert!(worklog.started.is_some());
}
