// Test comment version detection and format selection

use gouqi::AddComment;

#[test]
fn test_add_comment_v2_structure() {
    // Test that AddComment (V2) serializes correctly for V2 API
    let comment = AddComment::new("Test comment");
    let comment_json = serde_json::to_value(&comment).unwrap();

    // V2 API expects: { "body": "text", "visibility": ... }
    assert_eq!(comment_json["body"], "Test comment");
    assert!(
        comment_json["visibility"].is_null(),
        "visibility should be null when not set"
    );
}

#[test]
fn test_add_comment_v2_with_visibility() {
    use gouqi::Visibility;

    let visibility = Visibility {
        visibility_type: "role".to_string(),
        value: "Administrators".to_string(),
    };

    let comment = AddComment::new("Private comment").with_visibility(visibility);
    let comment_json = serde_json::to_value(&comment).unwrap();

    assert_eq!(comment_json["body"], "Private comment");
    assert!(comment_json["visibility"].is_object());
    assert_eq!(comment_json["visibility"]["type"], "role");
    assert_eq!(comment_json["visibility"]["value"], "Administrators");
}

#[test]
fn test_v2_spec_exact_format() {
    use gouqi::Visibility;

    // Verify exact JSON format matches V2 specification example
    let visibility = Visibility {
        visibility_type: "role".to_string(),
        value: "Administrators".to_string(),
    };

    let comment = AddComment::new("Comment text").with_visibility(visibility);
    let serialized = serde_json::to_value(&comment).unwrap();

    let expected = serde_json::json!({
        "body": "Comment text",
        "visibility": {
            "type": "role",
            "value": "Administrators"
        }
    });

    assert_eq!(
        serialized, expected,
        "AddComment must match V2 spec exactly"
    );
}

#[test]
fn test_comment_format_difference() {
    use gouqi::AddCommentAdf;

    // Demonstrate the key difference between V2 and V3 formats
    let v2_comment = AddComment::new("Test comment");
    let v3_comment = AddCommentAdf::from_text("Test comment");

    let v2_json = serde_json::to_value(&v2_comment).unwrap();
    let v3_json = serde_json::to_value(&v3_comment).unwrap();

    // V2: body is a string
    assert!(
        v2_json["body"].is_string(),
        "V2 body should be plain string"
    );

    // V3: body is an ADF document object
    assert!(v3_json["body"].is_object(), "V3 body should be ADF object");
    assert_eq!(v3_json["body"]["type"], "doc");
    assert_eq!(v3_json["body"]["version"], 1);
}
