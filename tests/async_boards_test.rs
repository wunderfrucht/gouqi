#[cfg(feature = "async")]
mod async_boards_tests {
    // No extern crate needed in Rust 2024 edition

    #[allow(unused_imports)]
    use futures::stream::StreamExt;
    use gouqi::r#async::Jira;
    use gouqi::{Credentials, Result, SearchOptions, SearchOptionsBuilder};

    #[tokio::test]
    async fn test_async_get_board() -> Result<()> {
        // Since we're not using a mock server with async tests, we'll just test
        // that the method can be called correctly

        // First test with a real client to ensure it compiles and works
        let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
        let _boards = jira.boards();

        // Just verify we can create an AsyncBoards instance without panic
        Ok(())
    }

    #[tokio::test]
    async fn test_async_list_boards() -> Result<()> {
        // Since we're not using a mock server with async tests, we'll just test
        // that the method can be called correctly

        let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
        let boards = jira.boards();
        let options = SearchOptions::default();

        // Just verify we can create the right method calls
        let _method_exists = boards.list(&options); // Not awaiting, just checking compilation

        Ok(())
    }

    #[tokio::test]
    async fn test_async_stream_boards() -> Result<()> {
        // Since we're not using a mock server with async tests, we'll just test
        // that the method can be called correctly

        let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
        let boards = jira.boards();
        let options = SearchOptionsBuilder::default()
            .start_at(0)
            .max_results(50)
            .build();

        // Just verify we can create the right method calls
        let _method_exists = boards.stream(&options); // Not awaiting, just checking compilation

        Ok(())
    }
}
