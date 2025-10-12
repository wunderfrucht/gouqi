//! Tests for TransitionTriggerOptionsBuilder comment helper with ADF support

use gouqi::TransitionTriggerOptions;

#[test]
fn test_transition_with_comment() {
    // Create a transition with a comment
    let options = TransitionTriggerOptions::builder("3")
        .comment("Resolved the issue")
        .build();

    // Serialize to JSON
    let serialized = serde_json::to_value(&options).unwrap();

    // Verify comment field exists and is ADF format
    let comment_field = &serialized["fields"]["comment"];
    assert!(comment_field.is_object(), "Comment field should exist");

    let body = &comment_field["body"];
    assert_eq!(body["type"], "doc");
    assert_eq!(body["version"], 1);

    // Verify content
    let content = body["content"].as_array().unwrap();
    assert_eq!(content[0]["type"], "paragraph");
    assert_eq!(content[0]["content"][0]["text"], "Resolved the issue");
}

#[test]
fn test_transition_with_multiline_comment() {
    // Create a transition with multi-line comment
    let options = TransitionTriggerOptions::builder("4")
        .comment("Fixed the bug\nTested locally\nReady for review")
        .build();

    let serialized = serde_json::to_value(&options).unwrap();
    let body = &serialized["fields"]["comment"]["body"];

    // Should have 3 paragraphs
    let content = body["content"].as_array().unwrap();
    assert_eq!(content.len(), 3);
    assert_eq!(content[0]["content"][0]["text"], "Fixed the bug");
    assert_eq!(content[1]["content"][0]["text"], "Tested locally");
    assert_eq!(content[2]["content"][0]["text"], "Ready for review");
}

#[test]
fn test_transition_with_comment_and_resolution() {
    // Create a transition with both comment and resolution
    let mut builder = TransitionTriggerOptions::builder("5");
    builder.comment("Issue resolved").resolution("Done");
    let options = builder.build();

    let serialized = serde_json::to_value(&options).unwrap();

    // Verify both fields exist
    let fields = &serialized["fields"];
    assert!(fields["comment"].is_object());
    assert!(fields["resolution"].is_object());

    // Verify comment is ADF
    assert_eq!(fields["comment"]["body"]["type"], "doc");

    // Verify resolution is correct
    assert_eq!(fields["resolution"]["name"], "Done");
}

#[test]
fn test_transition_without_comment() {
    // Create a transition without comment
    let options = TransitionTriggerOptions::builder("6").build();

    let serialized = serde_json::to_value(&options).unwrap();

    // Comment field should not exist
    assert!(serialized["fields"]["comment"].is_null());
}

#[test]
fn test_transition_comment_method_chaining() {
    // Test method chaining with comment
    let options = TransitionTriggerOptions::builder("7")
        .comment("Transitioning")
        .resolution("Fixed")
        .build();

    let serialized = serde_json::to_value(&options).unwrap();

    // Both fields should exist
    assert!(serialized["fields"]["comment"].is_object());
    assert!(serialized["fields"]["resolution"].is_object());
}

#[test]
fn test_transition_comment_empty_string() {
    // Test with empty comment
    let options = TransitionTriggerOptions::builder("8").comment("").build();

    let serialized = serde_json::to_value(&options).unwrap();
    let body = &serialized["fields"]["comment"]["body"];

    // Should still be valid ADF with empty paragraph
    assert_eq!(body["type"], "doc");
    assert_eq!(body["version"], 1);
    let content = body["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "paragraph");
}

#[test]
fn test_transition_comment_special_characters() {
    // Test with special characters
    let options = TransitionTriggerOptions::builder("9")
        .comment("Comment with <>&\"' chars")
        .build();

    let serialized = serde_json::to_value(&options).unwrap();
    let text = &serialized["fields"]["comment"]["body"]["content"][0]["content"][0]["text"];

    assert_eq!(text, "Comment with <>&\"' chars");
}
