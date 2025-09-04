//! Focused V3 API test coverage for specific missing areas
//!
//! This test file targets the exact coverage gaps identified.

use gouqi::{Credentials, Error, Jira, SearchApiVersion, SearchOptions};
use mockito::Server;

#[test]
fn test_validate_v3_empty_jql() {
    let server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();

    // Test V3 validation error for empty JQL
    let search_options = SearchOptions::builder().max_results(50).build();

    let result = jira.search().list("", &search_options); // Empty JQL should fail validation

    match result {
        Err(Error::InvalidQuery { message }) => {
            assert!(message.contains("empty JQL queries"));
        }
        _ => panic!("Expected InvalidQuery error for empty JQL"),
    }
}

#[test]
fn test_validate_v3_whitespace_only_jql() {
    let server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();

    // Test V3 validation error for whitespace-only JQL
    let search_options = SearchOptions::builder().max_results(50).build();

    let result = jira.search().list("   \n\t  ", &search_options); // Whitespace-only JQL

    match result {
        Err(Error::InvalidQuery { message }) => {
            assert!(message.contains("empty JQL queries"));
        }
        _ => panic!("Expected InvalidQuery error for whitespace-only JQL"),
    }
}

#[test]
fn test_search_options_field_state_tracking() {
    // Test fields_explicitly_set tracking
    let default_options = SearchOptions::default();
    assert!(!default_options.fields_explicitly_set());

    let explicit_fields = SearchOptions::builder().fields(vec!["id", "key"]).build();
    assert!(explicit_fields.fields_explicitly_set());

    // Test essential_fields() sets explicit flag
    let essential = SearchOptions::builder().essential_fields().build();
    assert!(essential.fields_explicitly_set());

    // Test standard_fields() sets explicit flag
    let standard = SearchOptions::builder().standard_fields().build();
    assert!(standard.fields_explicitly_set());

    // Test all_fields() sets explicit flag
    let all = SearchOptions::builder().all_fields().build();
    assert!(all.fields_explicitly_set());

    // Test minimal_fields() sets explicit flag
    let minimal = SearchOptions::builder().minimal_fields().build();
    assert!(minimal.fields_explicitly_set());
}

#[test]
fn test_search_options_builder_copy_operations() {
    // Test as_builder() method that copies SearchOptions to builder
    let original = SearchOptions::builder()
        .max_results(100)
        .fields(vec!["id", "key", "summary"])
        .validate(true)
        .build();

    // Test copying to builder preserves all settings
    let copied = original
        .as_builder()
        .max_results(200) // Override one setting
        .build();

    // Verify original fields were preserved but maxResults was overridden
    assert!(copied.fields_explicitly_set());
    assert_eq!(copied.max_results(), Some(200));
}

#[test]
fn test_builder_btree_map_ordering() {
    // Test that BTreeMap provides consistent parameter ordering
    let options1 = SearchOptions::builder()
        .max_results(50)
        .fields(vec!["id", "key"])
        .validate(true)
        .build();

    let options2 = SearchOptions::builder()
        .validate(true)
        .fields(vec!["id", "key"])
        .max_results(50)
        .build();

    // Both should serialize to the same parameter string due to BTreeMap ordering
    let serialized1 = options1.serialize().unwrap();
    let serialized2 = options2.serialize().unwrap();

    assert_eq!(serialized1, serialized2);

    // Parameters should be in alphabetical order
    assert!(serialized1.starts_with("fields="));
}

#[test]
fn test_v3_search_results_to_search_results_conversion() {
    use gouqi::{Issue, V3SearchResults};
    use serde_json::json;
    use std::collections::BTreeMap;

    // Create sample issues
    let mut fields1 = BTreeMap::new();
    fields1.insert("summary".to_string(), json!("First issue"));
    let issue1 = Issue {
        id: "1".to_string(),
        key: "TEST-1".to_string(),
        self_link: "http://test/1".to_string(),
        fields: fields1,
    };

    let mut fields2 = BTreeMap::new();
    fields2.insert("summary".to_string(), json!("Second issue"));
    let issue2 = Issue {
        id: "2".to_string(),
        key: "TEST-2".to_string(),
        self_link: "http://test/2".to_string(),
        fields: fields2,
    };

    // Test last page conversion
    let v3_last_page = V3SearchResults {
        issues: vec![issue1.clone(), issue2.clone()],
        is_last: true,
        next_page_token: None,
    };

    let search_results = v3_last_page.to_search_results(20, 50);

    // For last page, total should be exact: start_at + issues.len()
    assert_eq!(search_results.total, 22); // 20 + 2 issues
    assert_eq!(search_results.max_results, 50);
    assert_eq!(search_results.start_at, 20);
    assert_eq!(search_results.issues.len(), 2);
    assert_eq!(search_results.is_last_page, Some(true));
    assert_eq!(search_results.next_page_token, None);
    assert_eq!(search_results.total_is_accurate, Some(false)); // V3 never accurate

    // Test non-last page conversion
    let v3_middle_page = V3SearchResults {
        issues: vec![issue1.clone()],
        is_last: false,
        next_page_token: Some("next_token".to_string()),
    };

    let search_results = v3_middle_page.to_search_results(10, 25);

    // For middle page, total should estimate: start_at + issues.len() + max_results
    assert_eq!(search_results.total, 36); // 10 + 1 + 25
    assert_eq!(search_results.max_results, 25);
    assert_eq!(search_results.start_at, 10);
    assert_eq!(search_results.issues.len(), 1);
    assert_eq!(search_results.is_last_page, Some(false));
    assert_eq!(
        search_results.next_page_token,
        Some("next_token".to_string())
    );
    assert_eq!(search_results.total_is_accurate, Some(false));

    // Test empty results
    let v3_empty = V3SearchResults {
        issues: vec![],
        is_last: true,
        next_page_token: None,
    };

    let search_results = v3_empty.to_search_results(0, 50);
    assert_eq!(search_results.total, 0); // 0 + 0 issues
    assert_eq!(search_results.issues.len(), 0);
    assert_eq!(search_results.is_last_page, Some(true));
}

// Note: Async V3 validation test removed due to tokio runtime conflicts in test environment
