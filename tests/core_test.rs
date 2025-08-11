use gouqi::Credentials;

// Since core is not public, we'll be testing its functionality indirectly
// through the public interfaces of Jira and Async Jira.

#[cfg(test)]
mod core_tests {
    use super::*;
    use gouqi::Jira;

    // Test ClientCore::new via Jira::new
    #[test]
    fn test_client_core_new() {
        let jira = Jira::new("http://example.com", Credentials::Anonymous);
        assert!(jira.is_ok());
    }

    // Test ClientCore::new with invalid URL
    #[test]
    fn test_client_core_new_invalid_url() {
        let jira = Jira::new("not-a-url", Credentials::Anonymous);
        assert!(jira.is_err());
    }

    // Test ClientCore::new with different credential types
    #[test]
    fn test_client_core_credentials() {
        let jira_anon = Jira::new("http://example.com", Credentials::Anonymous);
        let jira_basic = Jira::new(
            "http://example.com",
            Credentials::Basic("user".to_string(), "pass".to_string()),
        );
        let jira_bearer = Jira::new(
            "http://example.com",
            Credentials::Bearer("token".to_string()),
        );
        let jira_cookie = Jira::new(
            "http://example.com",
            Credentials::Cookie("ABC123XYZ".to_string()),
        );

        // Just ensure they can be created without error
        assert!(jira_anon.is_ok());
        assert!(jira_basic.is_ok());
        assert!(jira_bearer.is_ok());
        assert!(jira_cookie.is_ok());
    }
}

// Testing the pagination trait
#[cfg(test)]
mod pagination_tests {
    use gouqi::core::PaginationInfo;

    // A dummy struct that implements PaginationInfo
    struct TestPagination;
    impl PaginationInfo for TestPagination {}

    #[test]
    fn test_more_pages() {
        // Test case 1: No more pages
        assert!(!TestPagination::more_pages(10, 90, 100));

        // Test case 2: More pages exist
        assert!(TestPagination::more_pages(10, 50, 100));

        // Test case 3: Exactly at the limit
        assert!(!TestPagination::more_pages(10, 90, 100));

        // Test case 4: Past the limit (shouldn't happen in practice)
        assert!(!TestPagination::more_pages(10, 100, 100));
    }
}

// Test the async client constructors
#[cfg(feature = "async")]
#[cfg(test)]
mod async_core_tests {
    use super::*;
    use gouqi::{Result, r#async::Jira as AsyncJira};

    #[test]
    fn test_async_client_core_new() {
        let jira = AsyncJira::new("http://example.com", Credentials::Anonymous);
        assert!(jira.is_ok());
    }

    #[test]
    fn test_async_client_with_cookie_auth() {
        let jira = AsyncJira::new(
            "http://example.com",
            Credentials::Cookie("ABC123XYZ".to_string()),
        );
        assert!(jira.is_ok());
    }

    #[test]
    fn test_conversion_between_sync_and_async() -> Result<()> {
        // Create a sync client
        let sync_jira = gouqi::Jira::new("http://example.com", Credentials::Anonymous)?;

        // Convert to async
        #[cfg(feature = "async")]
        let async_jira = sync_jira.into_async();

        // Convert back to sync
        #[cfg(feature = "async")]
        let _sync_jira2 = gouqi::sync::Jira::from(&async_jira);

        // Just ensure conversion works
        Ok(())
    }
}
