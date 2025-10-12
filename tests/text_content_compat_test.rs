//! Comprehensive backward compatibility tests for TextContent
//!
//! These tests ensure that the TextContent newtype provides ~95% backward compatibility
//! with the old String-based API through Deref and trait implementations.

use gouqi::TextContent;

#[test]
fn test_textcontent_deref_to_str() {
    let text = TextContent::from("Hello, World!");

    // Direct deref to &str
    let s: &str = &text;
    assert_eq!(s, "Hello, World!");

    // String methods work via Deref
    assert_eq!(text.len(), 13);
    assert!(text.contains("World"));
    assert!(text.starts_with("Hello"));
    assert!(text.ends_with("!"));
}

#[test]
fn test_textcontent_display() {
    let text = TextContent::from("Test Message");
    assert_eq!(format!("{}", text), "Test Message");
    assert_eq!(
        format!("{:?}", text),
        "TextContent { raw: String(\"Test Message\"), cached: \"Test Message\" }"
    );
}

#[test]
fn test_textcontent_string_comparison() {
    let text = TextContent::from("test");

    // Compare with &str
    assert_eq!(text, "test");
    assert!(text == "test");

    // Compare with String
    let test_string = String::from("test");
    assert_eq!(text, test_string);
    assert!(text == "test");

    // Compare with another TextContent
    let text2 = TextContent::from("test");
    assert_eq!(text, text2);
}

#[test]
fn test_textcontent_pattern_matching() {
    #[derive(serde::Deserialize)]
    struct TestComment {
        body: TextContent,
        author: String,
    }

    let json = r#"{"body": "Hello", "author": "Alice"}"#;
    let comment: TestComment = serde_json::from_str(json).unwrap();

    // Pattern matching works
    let TestComment { body, author } = comment;
    assert_eq!(&*body, "Hello");
    assert_eq!(author, "Alice");
}

#[test]
fn test_textcontent_as_ref() {
    let text = TextContent::from("reference test");

    // AsRef<str> works
    let s: &str = text.as_ref();
    assert_eq!(s, "reference test");

    // Can pass to functions expecting AsRef<str>
    fn takes_str_ref(s: impl AsRef<str>) -> usize {
        s.as_ref().len()
    }
    assert_eq!(takes_str_ref(&text), 14);
}

#[test]
fn test_textcontent_from_string() {
    // From String
    let text1 = TextContent::from(String::from("from string"));
    assert_eq!(&*text1, "from string");

    // From &str
    let text2 = TextContent::from("from str");
    assert_eq!(&*text2, "from str");

    // from_string method
    let text3 = TextContent::from_string("direct");
    assert_eq!(&*text3, "direct");
}

#[test]
fn test_textcontent_deserialize_string() {
    #[derive(serde::Deserialize)]
    struct TestStruct {
        content: TextContent,
    }

    // Deserialize from plain string (v2 API format)
    let json = r#"{"content": "Plain text message"}"#;
    let result: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(&*result.content, "Plain text message");
}

#[test]
fn test_textcontent_deserialize_adf() {
    #[derive(serde::Deserialize)]
    struct TestStruct {
        content: TextContent,
    }

    // Deserialize from ADF (v3 API format)
    let json = r#"{
        "content": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "First paragraph"}
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Second paragraph"}
                    ]
                }
            ]
        }
    }"#;

    let result: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(&*result.content, "First paragraph\nSecond paragraph");
}

#[test]
fn test_textcontent_serialize_preserves_raw() {
    // When deserializing and re-serializing, the raw value should be preserved
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestStruct {
        content: TextContent,
    }

    let json = r#"{"content":"Simple string"}"#;
    let parsed: TestStruct = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn test_comment_backward_compatibility() {
    use gouqi::Comment;

    // Old code pattern: accessing body field and using string methods
    let json = r#"{
        "self": "http://example.com/comment/1",
        "body": "Test comment"
    }"#;

    let comment: Comment = serde_json::from_str(json).unwrap();

    // OLD CODE PATTERNS THAT SHOULD STILL WORK:

    // 1. Direct field access
    let _body_ref = &comment.body;

    // 2. String methods via Deref
    assert_eq!(comment.body.len(), 12);
    assert!(comment.body.contains("Test"));

    // 3. Borrowing as &str
    let s: &str = &comment.body;
    assert_eq!(s, "Test comment");

    // 4. Printing
    let formatted = format!("{}", comment.body);
    assert_eq!(formatted, "Test comment");

    // 5. Comparison
    assert_eq!(comment.body, "Test comment");
}

#[test]
fn test_worklog_backward_compatibility() {
    use gouqi::Worklog;

    let json = r#"{
        "self": "http://example.com/worklog/1",
        "id": "1",
        "comment": "Work done",
        "timeSpentSeconds": 3600
    }"#;

    let worklog: Worklog = serde_json::from_str(json).unwrap();

    // OLD CODE PATTERNS THAT SHOULD STILL WORK:

    if let Some(ref comment) = worklog.comment {
        // 1. String methods via Deref
        assert_eq!(comment.len(), 9);
        assert!(comment.contains("done"));

        // 2. Borrowing as &str
        let s: &str = comment;
        assert_eq!(s, "Work done");

        // 3. Comparison
        assert_eq!(comment, "Work done");
    } else {
        panic!("Expected comment to be Some");
    }
}

#[test]
fn test_comment_with_adf_format() {
    use gouqi::Comment;

    // v3 API format with ADF
    let json = r#"{
        "self": "http://example.com/comment/1",
        "body": {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "ADF formatted comment"}
                    ]
                }
            ]
        }
    }"#;

    let comment: Comment = serde_json::from_str(json).unwrap();

    // Should extract text from ADF automatically
    assert_eq!(&*comment.body, "ADF formatted comment");
    assert!(comment.body.contains("formatted"));
}

#[test]
fn test_textcontent_clone() {
    let text1 = TextContent::from("clonable");
    let text2 = text1.clone();

    assert_eq!(text1, text2);
    assert_eq!(&*text2, "clonable");
}

#[test]
fn test_textcontent_borrow() {
    use std::borrow::Borrow;

    let text = TextContent::from("borrowed");
    let s: &str = text.borrow();
    assert_eq!(s, "borrowed");
}

#[test]
fn test_textcontent_empty_string() {
    let text = TextContent::from("");
    assert_eq!(&*text, "");
    assert_eq!(text.len(), 0);
    assert!(text.is_empty());
}

#[test]
fn test_textcontent_multiline() {
    let text = TextContent::from("Line 1\nLine 2\nLine 3");
    assert!(text.contains("Line 2"));
    assert_eq!(text.lines().count(), 3);
}

#[test]
fn test_textcontent_unicode() {
    let text = TextContent::from("Hello, 世界! 🌍");
    assert!(text.contains("世界"));
    assert!(text.contains("🌍"));
    assert_eq!(&*text, "Hello, 世界! 🌍");
}

#[test]
fn test_textcontent_eq_trait() {
    let text1 = TextContent::from("equal");
    let text2 = TextContent::from("equal");
    let text3 = TextContent::from("different");

    assert_eq!(text1, text2);
    assert_ne!(text1, text3);
}

#[test]
fn test_backwards_compat_only_breaks_on_ownership() {
    let text = TextContent::from("test");

    // These all work (95% of use cases):
    let _: &str = &text; // ✅
    let _ = text.len(); // ✅
    let _ = text.contains("t"); // ✅
    let _ = format!("{}", text); // ✅
    assert_eq!(text, "test"); // ✅

    // Only this breaks (5% of use cases):
    // let _: String = text;  // ❌ Type mismatch
    // But this works:
    let _: String = text.to_string(); // ✅
}
