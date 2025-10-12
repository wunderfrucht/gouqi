//! Tests for LinkComment ADF serialization

use gouqi::CreateIssueLinkInput;

#[test]
fn test_link_comment_serializes_to_adf() {
    // Create an issue link with a comment
    let link = CreateIssueLinkInput::new("Blocks", "TEST-1", "TEST-2")
        .with_comment("This blocks that issue");

    // Serialize to JSON
    let serialized = serde_json::to_value(&link).unwrap();

    // Verify the comment is serialized as ADF format
    let comment = &serialized["comment"];
    assert!(comment.is_object(), "Comment should be an object");

    let body = &comment["body"];
    assert!(body.is_object(), "Body should be an ADF object");
    assert_eq!(body["type"], "doc");
    assert_eq!(body["version"], 1);

    // Verify it has content
    let content = body["content"].as_array().unwrap();
    assert!(!content.is_empty(), "ADF content should not be empty");

    // First paragraph should contain our text
    let first_para = &content[0];
    assert_eq!(first_para["type"], "paragraph");

    let para_content = first_para["content"].as_array().unwrap();
    assert!(!para_content.is_empty());

    let text_node = &para_content[0];
    assert_eq!(text_node["type"], "text");
    assert_eq!(text_node["text"], "This blocks that issue");
}

#[test]
fn test_link_comment_multiline_serializes_to_adf() {
    // Create an issue link with multi-line comment
    let link = CreateIssueLinkInput::new("Relates", "TEST-3", "TEST-4")
        .with_comment("First line\nSecond line\nThird line");

    // Serialize to JSON
    let serialized = serde_json::to_value(&link).unwrap();

    let body = &serialized["comment"]["body"];
    assert_eq!(body["type"], "doc");

    // Should have 3 paragraphs
    let content = body["content"].as_array().unwrap();
    assert_eq!(content.len(), 3, "Should have 3 paragraphs");

    // Verify each paragraph has correct text
    assert_eq!(content[0]["content"][0]["text"], "First line");
    assert_eq!(content[1]["content"][0]["text"], "Second line");
    assert_eq!(content[2]["content"][0]["text"], "Third line");
}

#[test]
fn test_link_without_comment() {
    // Create an issue link without a comment
    let link = CreateIssueLinkInput::new("Blocks", "TEST-5", "TEST-6");

    // Serialize to JSON
    let serialized = serde_json::to_value(&link).unwrap();

    // Comment field should not be present
    assert!(serialized.get("comment").is_none());
}

#[test]
fn test_link_comment_empty_string() {
    // Create an issue link with empty comment
    let link = CreateIssueLinkInput::new("Blocks", "TEST-7", "TEST-8").with_comment("");

    // Serialize to JSON
    let serialized = serde_json::to_value(&link).unwrap();

    // Should still create ADF structure
    let body = &serialized["comment"]["body"];
    assert_eq!(body["type"], "doc");

    // Should have one empty paragraph
    let content = body["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "paragraph");
}

#[test]
fn test_link_comment_special_characters() {
    // Test with special characters
    let link = CreateIssueLinkInput::new("Duplicates", "TEST-9", "TEST-10")
        .with_comment("Comment with <>&\"' special chars");

    let serialized = serde_json::to_value(&link).unwrap();
    let text = &serialized["comment"]["body"]["content"][0]["content"][0]["text"];

    // Special characters should be preserved in the text
    assert_eq!(text, "Comment with <>&\"' special chars");
}

#[test]
fn test_link_comment_serialization_structure() {
    // Verify the overall structure matches expected v3 API format
    let link = CreateIssueLinkInput::new("Blocks", "PROJ-1", "PROJ-2").with_comment("Test comment");

    let serialized = serde_json::to_value(&link).unwrap();

    // Check top-level structure
    assert!(serialized["type"].is_object());
    assert_eq!(serialized["type"]["name"], "Blocks");
    assert_eq!(serialized["inwardIssue"]["key"], "PROJ-1");
    assert_eq!(serialized["outwardIssue"]["key"], "PROJ-2");

    // Check comment is ADF
    let comment_body = &serialized["comment"]["body"];
    assert_eq!(comment_body["version"], 1);
    assert_eq!(comment_body["type"], "doc");
    assert!(comment_body["content"].is_array());
}
