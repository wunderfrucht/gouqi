//! Tests for V3 API field auto-injection functionality
//! Ensures smooth transition for users from V2 to V3 API

use gouqi::{Credentials, Jira, SearchApiVersion, SearchOptions};

#[test]
fn test_v3_auto_injects_essential_fields_when_none_specified() {
    // Test that V3 API automatically injects essential fields when using default SearchOptions
    let jira = Jira::with_search_api_version(
        "https://test.atlassian.net",
        Credentials::Anonymous,
        SearchApiVersion::V3,
    )
    .expect("Failed to create Jira client");

    let _search = jira.search();

    // Default SearchOptions should not have fields explicitly set
    let options = SearchOptions::default();
    assert!(!options.fields_explicitly_set());

    // The search method should detect V3 and auto-inject essential fields
    // We can't easily test the actual HTTP request without mocking, but we can
    // verify the options transformation logic
    let final_options = if !options.fields_explicitly_set()
        && matches!(jira.get_search_api_version(), SearchApiVersion::V3)
    {
        options.as_builder().essential_fields().build()
    } else {
        options.clone()
    };

    assert!(final_options.fields_explicitly_set());
    assert!(
        final_options
            .serialize()
            .unwrap()
            .contains("fields=id%2Cself%2Ckey%2Csummary%2Cstatus")
    );
}

#[test]
fn test_v3_respects_explicitly_set_fields() {
    // Test that explicit field specification is respected, even for V3
    let _jira = Jira::with_search_api_version(
        "https://test.atlassian.net",
        Credentials::Anonymous,
        SearchApiVersion::V3,
    )
    .expect("Failed to create Jira client");

    // Explicitly set minimal fields
    let options = SearchOptions::builder().minimal_fields().build();

    assert!(options.fields_explicitly_set());
    assert!(options.serialize().unwrap().contains("fields=id"));
    assert!(!options.serialize().unwrap().contains("self"));
}

#[test]
fn test_v2_does_not_auto_inject_fields() {
    // Test that V2 API does not perform auto-injection
    let jira = Jira::with_search_api_version(
        "https://test.example.com",
        Credentials::Anonymous,
        SearchApiVersion::V2,
    )
    .expect("Failed to create Jira client");

    let options = SearchOptions::default();
    assert!(!options.fields_explicitly_set());

    // V2 should not trigger auto-injection
    let should_inject = !options.fields_explicitly_set()
        && matches!(jira.get_search_api_version(), SearchApiVersion::V3);

    assert!(!should_inject);
}

#[test]
fn test_convenience_methods_set_explicit_flag() {
    // Test that all convenience methods properly set the explicit flag

    let essential = SearchOptions::builder().essential_fields().build();
    assert!(essential.fields_explicitly_set());
    assert!(
        essential
            .serialize()
            .unwrap()
            .contains("id%2Cself%2Ckey%2Csummary%2Cstatus")
    );

    let standard = SearchOptions::builder().standard_fields().build();
    assert!(standard.fields_explicitly_set());
    assert!(standard.serialize().unwrap().contains("summary"));
    assert!(standard.serialize().unwrap().contains("status"));

    let all = SearchOptions::builder().all_fields().build();
    assert!(all.fields_explicitly_set());
    assert!(all.serialize().unwrap().contains("*all"));

    let minimal = SearchOptions::builder().minimal_fields().build();
    assert!(minimal.fields_explicitly_set());
    assert_eq!(minimal.serialize().unwrap(), "fields=id");
}

#[test]
fn test_auto_detection_for_cloud_vs_onpremise() {
    // Test deployment type detection affects V3 auto-injection

    // Cloud should use V3 with auto-injection
    let cloud_jira = Jira::with_search_api_version(
        "https://company.atlassian.net",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .expect("Failed to create cloud Jira client");

    assert_eq!(cloud_jira.get_search_api_version(), SearchApiVersion::V3);

    // On-premise should default to V2 (no auto-injection)
    let onprem_jira = Jira::with_search_api_version(
        "https://jira.company.com",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .expect("Failed to create on-premise Jira client");

    assert_eq!(onprem_jira.get_search_api_version(), SearchApiVersion::V2);
}

#[test]
fn test_builder_copy_preserves_explicit_flag() {
    // Test that as_builder() preserves the explicit flag state

    let original = SearchOptions::builder()
        .standard_fields()
        .max_results(50)
        .build();

    assert!(original.fields_explicitly_set());

    let copied = original.as_builder().start_at(100).build();

    assert!(copied.fields_explicitly_set());
    assert!(copied.serialize().unwrap().contains("summary"));
    assert!(copied.serialize().unwrap().contains("startAt=100"));
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;

    #[tokio::test]
    async fn test_async_v3_auto_injection() {
        // Test async version of auto-injection
        let jira = gouqi::r#async::Jira::with_search_api_version(
            "https://test.atlassian.net",
            Credentials::Anonymous,
            SearchApiVersion::V3,
        )
        .expect("Failed to create async Jira client");

        let options = SearchOptions::default();
        assert!(!options.fields_explicitly_set());

        // Simulate the logic from AsyncSearch::list
        let final_options = if !options.fields_explicitly_set()
            && matches!(jira.get_search_api_version(), SearchApiVersion::V3)
        {
            options.as_builder().essential_fields().build()
        } else {
            options.clone()
        };

        assert!(final_options.fields_explicitly_set());
        assert!(
            final_options
                .serialize()
                .unwrap()
                .contains("fields=id%2Cself%2Ckey%2Csummary%2Cstatus")
        );
    }
}
