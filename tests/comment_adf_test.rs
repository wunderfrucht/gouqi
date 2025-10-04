// Test ADF comment structure serialization against V3 specification

use gouqi::{AddCommentAdf, AdfDocument};
use serde_json::json;

#[test]
fn test_adf_document_single_line() {
    // Test that ADF structure serializes correctly according to V3 spec
    let adf = AdfDocument::from_text("Hello world");
    let serialized = serde_json::to_value(&adf).unwrap();

    // Expected structure per V3 spec:
    // {
    //   "version": 1,
    //   "type": "doc",
    //   "content": [
    //     {
    //       "type": "paragraph",
    //       "content": [
    //         {
    //           "type": "text",
    //           "text": "Hello world"
    //         }
    //       ]
    //     }
    //   ]
    // }

    assert_eq!(serialized["version"], 1, "ADF version must be 1");
    assert_eq!(serialized["type"], "doc", "Root type must be 'doc'");
    assert!(serialized["content"].is_array(), "content must be an array");

    let para = &serialized["content"][0];
    assert_eq!(para["type"], "paragraph", "First node must be paragraph");

    let text_node = &para["content"].as_array().unwrap()[0];
    assert_eq!(text_node["type"], "text", "Inner node must be text");
    assert_eq!(text_node["text"], "Hello world");
}

#[test]
fn test_adf_document_multiline() {
    // Test multiline text creates separate paragraphs
    let multiline = AdfDocument::from_text("Line 1\nLine 2\nLine 3");
    let multiline_json = serde_json::to_value(&multiline).unwrap();

    assert_eq!(multiline_json["version"], 1);
    assert_eq!(multiline_json["type"], "doc");

    let paragraphs = multiline_json["content"].as_array().unwrap();
    assert_eq!(paragraphs.len(), 3, "Should have 3 paragraphs for 3 lines");

    // Check each paragraph
    assert_eq!(paragraphs[0]["type"], "paragraph");
    assert_eq!(paragraphs[0]["content"][0]["text"], "Line 1");

    assert_eq!(paragraphs[1]["type"], "paragraph");
    assert_eq!(paragraphs[1]["content"][0]["text"], "Line 2");

    assert_eq!(paragraphs[2]["type"], "paragraph");
    assert_eq!(paragraphs[2]["content"][0]["text"], "Line 3");
}

#[test]
fn test_adf_document_empty() {
    // Empty document should have at least one empty paragraph
    let empty = AdfDocument::from_text("");
    let empty_json = serde_json::to_value(&empty).unwrap();

    assert_eq!(empty_json["version"], 1);
    assert_eq!(empty_json["type"], "doc");

    let paragraphs = empty_json["content"].as_array().unwrap();
    assert_eq!(
        paragraphs.len(),
        1,
        "Empty text should create one paragraph"
    );
    assert_eq!(paragraphs[0]["type"], "paragraph");

    // Empty paragraph has no content
    let para_content = paragraphs[0]["content"].as_array();
    assert!(
        para_content.is_none() || para_content.unwrap().is_empty(),
        "Empty paragraph should have no content"
    );
}

#[test]
fn test_add_comment_adf_structure() {
    // Test AddCommentAdf serializes correctly for V3 API
    let comment = AddCommentAdf::from_text("Test comment");
    let comment_json = serde_json::to_value(&comment).unwrap();

    // V3 API expects: { "body": { ADF document }, "visibility": ... }
    assert!(comment_json["body"].is_object(), "body must be an object");
    assert_eq!(comment_json["body"]["version"], 1);
    assert_eq!(comment_json["body"]["type"], "doc");

    // Visibility should be omitted when not set
    assert!(comment_json["visibility"].is_null());
}

#[test]
fn test_add_comment_adf_with_visibility() {
    use gouqi::Visibility;

    let visibility = Visibility {
        visibility_type: "role".to_string(),
        value: "Administrators".to_string(),
    };

    let comment = AddCommentAdf::from_text("Private comment").with_visibility(visibility);
    let comment_json = serde_json::to_value(&comment).unwrap();

    assert!(comment_json["body"].is_object());
    assert!(comment_json["visibility"].is_object());
    assert_eq!(comment_json["visibility"]["type"], "role");
    assert_eq!(comment_json["visibility"]["value"], "Administrators");
}

#[test]
fn test_v3_spec_exact_format() {
    // Verify exact JSON format matches V3 specification example
    let adf = AdfDocument::from_text("comment text here");
    let serialized = serde_json::to_value(&adf).unwrap();

    let expected = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "comment text here"
                    }
                ]
            }
        ]
    });

    assert_eq!(serialized, expected, "ADF must match V3 spec exactly");
}
