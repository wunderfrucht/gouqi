//! Comprehensive tests for Issue.environment() with both v2 string and v3 ADF formats

use gouqi::{AdfDocument, Issue};
use serde_json::json;
use std::collections::BTreeMap;

fn create_issue_with_environment(environment: serde_json::Value) -> Issue {
    let mut fields = BTreeMap::new();
    fields.insert("environment".to_string(), environment);

    Issue {
        self_link: "http://jira.example.com/rest/api/2/issue/TEST-1".to_string(),
        key: "TEST-1".to_string(),
        id: "10000".to_string(),
        fields,
    }
}

#[test]
fn test_environment_v2_string_format() {
    // Test v2 API format where environment is a plain string
    let issue = create_issue_with_environment(json!("Windows 10, Chrome 120"));

    assert_eq!(
        issue.environment(),
        Some("Windows 10, Chrome 120".to_string())
    );
}

#[test]
fn test_environment_adf_format_simple() {
    // Test v3 API format where environment is ADF with single paragraph
    let adf = AdfDocument::from_text("Production: Ubuntu 22.04 LTS");

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    assert_eq!(
        issue.environment(),
        Some("Production: Ubuntu 22.04 LTS".to_string())
    );
}

#[test]
fn test_environment_adf_format_multiline() {
    // Test v3 ADF with multiple paragraphs
    let text = "OS: Windows 11\nBrowser: Firefox 121\nJava: OpenJDK 17";
    let adf = AdfDocument::from_text(text);

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    let environment = issue.environment().unwrap();
    assert_eq!(environment, text);

    // Verify it has multiple lines
    assert_eq!(environment.lines().count(), 3);
}

#[test]
fn test_environment_adf_format_empty() {
    // Test v3 ADF with empty content
    let adf = AdfDocument::from_text("");

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    // Empty ADF should return None
    assert!(issue.environment().is_none());
}

#[test]
fn test_environment_no_field() {
    // Test issue without environment field
    let issue = Issue {
        self_link: "http://jira.example.com/rest/api/2/issue/TEST-1".to_string(),
        key: "TEST-1".to_string(),
        id: "10000".to_string(),
        fields: BTreeMap::new(),
    };

    assert!(issue.environment().is_none());
}

#[test]
fn test_environment_adf_with_formatting() {
    // Test ADF with formatting marks (should be ignored in plain text extraction)
    let environment_json = json!({
        "version": 1,
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Server: "
                    },
                    {
                        "type": "text",
                        "text": "production-01",
                        "marks": [{"type": "code"}]
                    }
                ]
            }
        ]
    });

    let issue = create_issue_with_environment(environment_json);

    assert_eq!(
        issue.environment(),
        Some("Server: production-01".to_string())
    );
}

#[test]
fn test_environment_adf_with_nested_content() {
    // Test ADF with nested structures
    let environment_json = json!({
        "version": 1,
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Operating System: macOS 14"
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Architecture: ARM64"
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Memory: 16GB"
                    }
                ]
            }
        ]
    });

    let issue = create_issue_with_environment(environment_json);

    let environment = issue.environment().unwrap();
    assert_eq!(
        environment,
        "Operating System: macOS 14\nArchitecture: ARM64\nMemory: 16GB"
    );
}

#[test]
fn test_environment_special_characters() {
    // Test v2 format with special characters
    let issue = create_issue_with_environment(json!("OS: Linux & macOS (x86_64)"));

    assert_eq!(
        issue.environment(),
        Some("OS: Linux & macOS (x86_64)".to_string())
    );
}

#[test]
fn test_environment_unicode() {
    // Test v3 ADF with Unicode characters
    let text = "环境: 生产环境\nサーバー: Tokyo";
    let adf = AdfDocument::from_text(text);

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    assert_eq!(issue.environment(), Some(text.to_string()));
}

#[test]
fn test_environment_very_long_text() {
    // Test v3 ADF with very long text
    let long_text = "Environment Details: ".to_string() + &"x".repeat(5000);
    let adf = AdfDocument::from_text(&long_text);

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    let environment = issue.environment().unwrap();
    assert_eq!(environment.len(), long_text.len());
    assert_eq!(environment, long_text);
}

#[test]
fn test_environment_whitespace_handling() {
    // Test v2 format with various whitespace
    let issue = create_issue_with_environment(json!("  Leading spaces\n\nBlank line\n"));

    // Should preserve whitespace as-is in v2
    assert_eq!(
        issue.environment(),
        Some("  Leading spaces\n\nBlank line\n".to_string())
    );
}

#[test]
fn test_environment_adf_round_trip() {
    // Test that ADF conversion is consistent
    let original_text = "Server: prod-server-01\nRegion: us-east-1\nVersion: 2.5.0";
    let adf = AdfDocument::from_text(original_text);

    // Convert to JSON and back
    let adf_json = serde_json::to_value(&adf).unwrap();
    let issue = create_issue_with_environment(adf_json);

    // Should extract the same text
    assert_eq!(issue.environment(), Some(original_text.to_string()));
}

#[test]
fn test_environment_with_complex_structure() {
    // Test realistic environment description
    let text = "Browser: Chrome 120.0.6099.109\nOS: Windows 11 Pro (Build 22621)\nScreen Resolution: 1920x1080\nMemory: 32GB DDR5\nCPU: AMD Ryzen 9 7950X";
    let adf = AdfDocument::from_text(text);

    let issue = create_issue_with_environment(serde_json::to_value(&adf).unwrap());

    let environment = issue.environment().unwrap();
    assert_eq!(environment, text);
    assert_eq!(environment.lines().count(), 5);
}

#[test]
fn test_environment_both_description_and_environment() {
    // Test that environment and description work independently
    let mut fields = BTreeMap::new();
    fields.insert("description".to_string(), json!("This is the description"));
    fields.insert("environment".to_string(), json!("This is the environment"));

    let issue = Issue {
        self_link: "http://jira.example.com/rest/api/2/issue/TEST-1".to_string(),
        key: "TEST-1".to_string(),
        id: "10000".to_string(),
        fields,
    };

    assert_eq!(
        issue.description(),
        Some("This is the description".to_string())
    );
    assert_eq!(
        issue.environment(),
        Some("This is the environment".to_string())
    );
}
