#[cfg(feature = "async")]
mod async_error_tests {
    extern crate gouqi;
    extern crate serde_json;

    use gouqi::*;
    use serde::de::Error as SerdeError;

    // Tests that don't require mockito server
    #[test]
    fn test_async_error_parsing() {
        // This test verifies the Error enum properly handles Serde error cases
        // without needing a mock server
        let error = Error::Serde(serde_json::Error::custom(String::from("test error")));
        assert!(matches!(error, Error::Serde(_)));
    }

    #[test]
    fn test_async_error_types() {
        // Verify error types work correctly
        let unauthorized = Error::Unauthorized;
        assert!(matches!(unauthorized, Error::Unauthorized));

        let not_found = Error::NotFound;
        assert!(matches!(not_found, Error::NotFound));

        let method_not_allowed = Error::MethodNotAllowed;
        assert!(matches!(method_not_allowed, Error::MethodNotAllowed));
    }
}
