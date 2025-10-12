//! Comprehensive tests for Comment.body() with both v2 string and v3 ADF formats
//! Tests the fix for Comment body handling in JIRA v3 API

use gouqi::{AdfDocument, Comment};
use serde_json::json;

#[test]
fn test_comment_body_v2_string_format() {
    // Test v2 API format where body is a plain string
    let comment_json = json!({
        "id": "10000",
        "self": "http://jira.example.com/rest/api/2/comment/10000",
        "body": "This is a plain text comment from v2 API",
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    assert_eq!(comment.id, Some("10000".to_string()));
    assert_eq!(
        comment.body(),
        Some("This is a plain text comment from v2 API".to_string())
    );
}

#[test]
fn test_comment_body_adf_format_simple() {
    // Test v3 API format where body is ADF with single paragraph
    let adf = AdfDocument::from_text("This is an ADF comment from v3 API");

    let comment_json = json!({
        "id": "10001",
        "self": "http://jira.example.com/rest/api/3/comment/10001",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    assert_eq!(comment.id, Some("10001".to_string()));
    assert_eq!(
        comment.body(),
        Some("This is an ADF comment from v3 API".to_string())
    );
}

#[test]
fn test_comment_body_adf_format_multiline() {
    // Test v3 ADF with multiple paragraphs
    let text = "First paragraph\nSecond paragraph\nThird paragraph";
    let adf = AdfDocument::from_text(text);

    let comment_json = json!({
        "id": "10002",
        "self": "http://jira.example.com/rest/api/3/comment/10002",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    let body = comment.body().unwrap();
    assert_eq!(body, text);

    // Verify it has multiple lines
    assert_eq!(body.lines().count(), 3);
}

#[test]
fn test_comment_body_adf_format_empty() {
    // Test v3 ADF with empty content
    let adf = AdfDocument::from_text("");

    let comment_json = json!({
        "id": "10003",
        "self": "http://jira.example.com/rest/api/3/comment/10003",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    // Empty ADF should return None (no meaningful content)
    assert!(comment.body().is_none());
}

#[test]
fn test_comment_body_adf_with_inline_formatting() {
    // Test ADF with bold, italic marks (should be ignored in plain text extraction)
    let comment_json = json!({
        "id": "10004",
        "self": "http://jira.example.com/rest/api/3/comment/10004",
        "body": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "This is "
                        },
                        {
                            "type": "text",
                            "text": "bold",
                            "marks": [{"type": "strong"}]
                        },
                        {
                            "type": "text",
                            "text": " and "
                        },
                        {
                            "type": "text",
                            "text": "italic",
                            "marks": [{"type": "em"}]
                        },
                        {
                            "type": "text",
                            "text": " text"
                        }
                    ]
                }
            ]
        },
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    // Should extract plain text without formatting marks
    assert_eq!(
        comment.body(),
        Some("This is bold and italic text".to_string())
    );
}

#[test]
fn test_comment_body_adf_with_nested_content() {
    // Test ADF with nested structures
    let comment_json = json!({
        "id": "10005",
        "self": "http://jira.example.com/rest/api/3/comment/10005",
        "body": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Outer text "
                        },
                        {
                            "type": "text",
                            "text": "with nested",
                            "marks": [{"type": "code"}]
                        }
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Second paragraph"
                        }
                    ]
                }
            ]
        },
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    let body = comment.body().unwrap();
    assert_eq!(body, "Outer text with nested\nSecond paragraph");
}

#[test]
fn test_comment_body_special_characters() {
    // Test v2 format with special characters
    let comment_json = json!({
        "id": "10006",
        "self": "http://jira.example.com/rest/api/2/comment/10006",
        "body": "Comment with special chars: <>&\"'",
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    assert_eq!(
        comment.body(),
        Some("Comment with special chars: <>&\"'".to_string())
    );
}

#[test]
fn test_comment_body_unicode() {
    // Test v3 ADF with Unicode characters
    let text = "Unicode test: 你好 🎉 Ñoño";
    let adf = AdfDocument::from_text(text);

    let comment_json = json!({
        "id": "10007",
        "self": "http://jira.example.com/rest/api/3/comment/10007",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    assert_eq!(comment.body(), Some(text.to_string()));
}

#[test]
fn test_comment_body_very_long_text() {
    // Test v3 ADF with very long text (performance test)
    let long_text = "A".repeat(10000);
    let adf = AdfDocument::from_text(&long_text);

    let comment_json = json!({
        "id": "10008",
        "self": "http://jira.example.com/rest/api/3/comment/10008",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    let body = comment.body().unwrap();
    assert_eq!(body.len(), 10000);
    assert_eq!(body, long_text);
}

#[test]
fn test_comment_body_many_paragraphs() {
    // Test v3 ADF with many paragraphs
    let mut paragraphs = Vec::new();
    for i in 0..50 {
        paragraphs.push(format!("Paragraph {}", i));
    }
    let text = paragraphs.join("\n");
    let adf = AdfDocument::from_text(&text);

    let comment_json = json!({
        "id": "10009",
        "self": "http://jira.example.com/rest/api/3/comment/10009",
        "body": adf,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    let body = comment.body().unwrap();
    assert_eq!(body, text);
    assert_eq!(body.lines().count(), 50);
}

#[test]
fn test_comment_body_raw_field_access() {
    // Test direct access to body_raw field for advanced use cases
    let comment_json = json!({
        "id": "10010",
        "self": "http://jira.example.com/rest/api/2/comment/10010",
        "body": "Direct access test",
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    // Verify body_raw contains the raw value
    assert!(comment.body_raw.is_string());
    assert_eq!(comment.body_raw.as_str(), Some("Direct access test"));

    // And that body() extracts it correctly
    assert_eq!(comment.body(), Some("Direct access test".to_string()));
}

#[test]
fn test_comment_body_adf_round_trip() {
    // Test that ADF conversion is consistent
    let original_text = "Line 1\nLine 2\nLine 3";
    let adf = AdfDocument::from_text(original_text);

    // Convert to JSON and back
    let adf_json = serde_json::to_value(&adf).unwrap();

    let comment_json = json!({
        "id": "10011",
        "self": "http://jira.example.com/rest/api/3/comment/10011",
        "body": adf_json,
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    // Should extract the same text
    assert_eq!(comment.body(), Some(original_text.to_string()));
}

#[test]
fn test_comment_with_optional_fields() {
    // Test Comment with only required fields
    let comment_json = json!({
        "self": "http://jira.example.com/rest/api/2/comment/10012",
        "body": "Minimal comment"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    assert!(comment.id.is_none());
    assert!(comment.author.is_none());
    assert!(comment.update_author.is_none());
    assert!(comment.created.is_none());
    assert!(comment.updated.is_none());
    assert!(comment.visibility.is_none());
    assert_eq!(comment.body(), Some("Minimal comment".to_string()));
}

#[test]
fn test_comment_body_whitespace_handling() {
    // Test v2 format with various whitespace
    let comment_json = json!({
        "id": "10013",
        "self": "http://jira.example.com/rest/api/2/comment/10013",
        "body": "  Leading and trailing spaces  \n\nWith blank lines\n",
        "created": "2024-01-01T10:00:00.000+0000",
        "updated": "2024-01-01T10:00:00.000+0000"
    });

    let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();

    // Should preserve whitespace as-is in v2
    assert_eq!(
        comment.body(),
        Some("  Leading and trailing spaces  \n\nWith blank lines\n".to_string())
    );
}
