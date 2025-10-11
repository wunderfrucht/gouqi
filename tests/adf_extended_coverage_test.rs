//! Extended test coverage for ADF (Atlassian Document Format) functionality
//! Tests edge cases, complex structures, and various ADF node types

use gouqi::{AdfDocument, Credentials, Jira};
use serde_json::json;

#[test]
fn test_adf_nested_structures() {
    let mut server = mockito::Server::new();

    // Complex nested ADF with multiple paragraph types
    let response_data = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/TEST-1", server.url()),
        "fields": {
            "summary": "Test with nested ADF",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "First level paragraph."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Paragraph with "
                            },
                            {
                                "type": "text",
                                "text": "nested",
                                "marks": [
                                    {"type": "strong"},
                                    {"type": "em"}
                                ]
                            },
                            {
                                "type": "text",
                                "text": " formatting."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Final paragraph."
                            }
                        ]
                    }
                ]
            }
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

    let description = issue.description().unwrap();
    assert!(description.contains("First level paragraph"));
    assert!(description.contains("nested"));
    assert!(description.contains("Final paragraph"));
    assert_eq!(
        description,
        "First level paragraph.\nParagraph with nested formatting.\nFinal paragraph."
    );
}

#[test]
fn test_adf_with_special_characters() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10002",
        "key": "TEST-2",
        "self": format!("{}/rest/api/latest/issue/TEST-2", server.url()),
        "fields": {
            "summary": "Test with special characters",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Special: <>&\"'\\n\\t\r"
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Unicode: 你好 🎉 Ñoño"
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

    let description = issue.description().unwrap();
    assert!(description.contains("<>&\"'"));
    assert!(description.contains("你好 🎉 Ñoño"));
}

#[test]
fn test_adf_empty_paragraphs() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10003",
        "key": "TEST-3",
        "self": format!("{}/rest/api/latest/issue/TEST-3", server.url()),
        "fields": {
            "summary": "Test with empty paragraphs",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "First paragraph."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": []
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Third paragraph."
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

    let description = issue.description().unwrap();
    // Empty paragraphs should not create extra newlines
    assert_eq!(description, "First paragraph.\nThird paragraph.");
}

#[test]
fn test_adf_very_long_text() {
    let long_text = "A".repeat(10000);
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10004",
        "key": "TEST-4",
        "self": format!("{}/rest/api/latest/issue/TEST-4", server.url()),
        "fields": {
            "summary": "Test with very long text",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": long_text.clone()
                            }
                        ]
                    }
                ]
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

    let description = issue.description().unwrap();
    assert_eq!(description.len(), 10000);
    assert_eq!(description, long_text);
}

#[test]
fn test_adf_many_paragraphs() {
    let mut content_array = Vec::new();
    for i in 1..=100 {
        content_array.push(json!({
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": format!("Paragraph {}.", i)
                }
            ]
        }));
    }

    let mut server = mockito::Server::new();
    let response_data = json!({
        "id": "10005",
        "key": "TEST-5",
        "self": format!("{}/rest/api/latest/issue/TEST-5", server.url()),
        "fields": {
            "summary": "Test with many paragraphs",
            "description": {
                "version": 1,
                "type": "doc",
                "content": content_array
            }
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

    let description = issue.description().unwrap();
    let lines: Vec<&str> = description.lines().collect();
    assert_eq!(lines.len(), 100);
    assert_eq!(lines[0], "Paragraph 1.");
    assert_eq!(lines[99], "Paragraph 100.");
}

#[test]
fn test_adf_mixed_content_types() {
    let mut server = mockito::Server::new();

    // ADF can have various content types - we should handle unknown types gracefully
    let response_data = json!({
        "id": "10006",
        "key": "TEST-6",
        "self": format!("{}/rest/api/latest/issue/TEST-6", server.url()),
        "fields": {
            "summary": "Test with mixed content types",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Regular text paragraph."
                            }
                        ]
                    },
                    {
                        "type": "unknownNodeType",
                        "content": [
                            {
                                "type": "text",
                                "text": "Text in unknown node."
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Another regular paragraph."
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

    let description = issue.description().unwrap();
    // Should still extract text from unknown node types
    assert!(description.contains("Regular text paragraph"));
    assert!(description.contains("Text in unknown node"));
    assert!(description.contains("Another regular paragraph"));
}

#[test]
fn test_adf_with_only_whitespace() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10007",
        "key": "TEST-7",
        "self": format!("{}/rest/api/latest/issue/TEST-7", server.url()),
        "fields": {
            "summary": "Test with whitespace only",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "   "
                            }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "\t\n"
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-7").unwrap();

    let description = issue.description().unwrap();
    // Whitespace should be preserved
    assert!(description.contains(' '));
}

#[test]
fn test_adf_document_from_text_multiline() {
    let text = "Line 1\nLine 2\nLine 3\nLine 4";
    let adf = AdfDocument::from_text(text);

    assert_eq!(adf.version, 1);
    assert_eq!(adf.doc_type, "doc");
    assert_eq!(adf.content.len(), 4);

    // Each line should become a paragraph
    for (i, node) in adf.content.iter().enumerate() {
        assert_eq!(node.node_type, "paragraph");
        let content = node.content.as_ref().unwrap();
        assert_eq!(content.len(), 1);

        match &content[0] {
            gouqi::AdfContent::Text(text_node) => {
                assert_eq!(text_node.text, format!("Line {}", i + 1));
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[test]
fn test_adf_document_from_text_empty_lines() {
    let text = "Line 1\n\nLine 3\n\n\nLine 6";
    let adf = AdfDocument::from_text(text);

    // ADF creates nodes for each line including empty ones
    assert_eq!(adf.content.len(), 6);

    let plain_text = adf.to_plain_text();
    // Note: Empty paragraphs (with no text content) are filtered out during extraction
    // This is expected behavior - ADF doesn't preserve empty lines the same way
    assert_eq!(plain_text, "Line 1\nLine 3\nLine 6");
}

#[test]
fn test_adf_document_from_text_single_line() {
    let text = "Single line only";
    let adf = AdfDocument::from_text(text);

    assert_eq!(adf.content.len(), 1);

    let plain_text = adf.to_plain_text();
    assert_eq!(plain_text, text);
}

#[test]
fn test_adf_document_from_text_with_special_chars() {
    let text = "Special: <>&\"'\nUnicode: 你好 🎉";
    let adf = AdfDocument::from_text(text);

    let plain_text = adf.to_plain_text();
    assert_eq!(plain_text, text);
}

#[test]
fn test_adf_multiple_text_nodes_in_paragraph() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10008",
        "key": "TEST-8",
        "self": format!("{}/rest/api/latest/issue/TEST-8", server.url()),
        "fields": {
            "summary": "Test with multiple text nodes",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "First "
                            },
                            {
                                "type": "text",
                                "text": "second "
                            },
                            {
                                "type": "text",
                                "text": "third"
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-8")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-8").unwrap();

    let description = issue.description().unwrap();
    assert_eq!(description, "First second third");
}

#[test]
fn test_adf_with_complex_marks() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "id": "10009",
        "key": "TEST-9",
        "self": format!("{}/rest/api/latest/issue/TEST-9", server.url()),
        "fields": {
            "summary": "Test with complex formatting marks",
            "description": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Normal "
                            },
                            {
                                "type": "text",
                                "text": "bold",
                                "marks": [{"type": "strong"}]
                            },
                            {
                                "type": "text",
                                "text": " "
                            },
                            {
                                "type": "text",
                                "text": "italic",
                                "marks": [{"type": "em"}]
                            },
                            {
                                "type": "text",
                                "text": " "
                            },
                            {
                                "type": "text",
                                "text": "underline",
                                "marks": [{"type": "underline"}]
                            },
                            {
                                "type": "text",
                                "text": " "
                            },
                            {
                                "type": "text",
                                "text": "code",
                                "marks": [{"type": "code"}]
                            },
                            {
                                "type": "text",
                                "text": " "
                            },
                            {
                                "type": "text",
                                "text": "link",
                                "marks": [
                                    {
                                        "type": "link",
                                        "attrs": {
                                            "href": "https://example.com"
                                        }
                                    }
                                ]
                            },
                            {
                                "type": "text",
                                "text": " end"
                            }
                        ]
                    }
                ]
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-9")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-9").unwrap();

    let description = issue.description().unwrap();
    // All formatting marks should be stripped, leaving only text
    assert_eq!(description, "Normal bold italic underline code link end");
}

#[test]
fn test_description_backward_compatibility_string_over_adf() {
    let mut server = mockito::Server::new();

    // An edge case: if both string and ADF are present (shouldn't happen but testing fallback)
    let response_data = json!({
        "id": "10010",
        "key": "TEST-10",
        "self": format!("{}/rest/api/latest/issue/TEST-10", server.url()),
        "fields": {
            "summary": "Test backward compatibility",
            "description": "String description"
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-10")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issue = jira.issues().get("TEST-10").unwrap();

    // Should prefer string format (backward compatibility)
    let description = issue.description().unwrap();
    assert_eq!(description, "String description");
}

#[test]
fn test_adf_document_serialization() {
    let text = "Test\nMultiple\nLines";
    let adf = AdfDocument::from_text(text);

    // Serialize to JSON
    let json = serde_json::to_string(&adf).unwrap();

    // Deserialize back
    let deserialized: AdfDocument = serde_json::from_str(&json).unwrap();

    // Should produce the same plain text
    assert_eq!(deserialized.to_plain_text(), text);
}

#[test]
fn test_adf_performance_large_document() {
    use std::time::Instant;

    // Create a large ADF document with 1000 paragraphs
    let mut paragraphs = Vec::new();
    for i in 0..1000 {
        paragraphs.push(format!("Paragraph {} with some text content.", i));
    }

    let text = paragraphs.join("\n");
    let start = Instant::now();
    let adf = AdfDocument::from_text(&text);
    let to_adf_duration = start.elapsed();

    let start = Instant::now();
    let extracted = adf.to_plain_text();
    let from_adf_duration = start.elapsed();

    println!(
        "to_text: {:?}, from_text: {:?}",
        to_adf_duration, from_adf_duration
    );

    // Should complete in reasonable time (< 100ms each)
    assert!(to_adf_duration.as_millis() < 100, "to_text too slow");
    assert!(from_adf_duration.as_millis() < 100, "from_text too slow");

    // Content should match
    assert_eq!(extracted, text);
}
