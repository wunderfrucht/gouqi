#[cfg(feature = "async")]
mod async_tests {
    // No extern crate needed in Rust 2024 edition

    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::*;
    use serde::Serialize;

    const JIRA_HOST: &str = "http://jira.com";

    #[derive(Serialize, Debug, Default)]
    struct EmptyBody;

    #[test]
    fn async_jira_new_should_err_if_no_uri() {
        let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
        let jira = AsyncJira::new("12345", credentials);
        assert!(jira.is_err());
    }

    #[test]
    fn async_jira_new_should_ok_with_uri() {
        let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
        let jira = AsyncJira::new(JIRA_HOST, credentials);
        assert!(jira.is_ok());
    }

    #[tokio::test]
    async fn async_jira_http_delete() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("DELETE", "/rest/api/latest/endpoint")
            .with_status(201)
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        jira.delete::<EmptyResponse>("api", "/endpoint")
            .await
            .unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_get_bearer() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/endpoint")
            .with_status(201)
            .match_header("authorization", "Bearer 12345")
            .create_async()
            .await;

        // Run the test
        let credentials = Credentials::Bearer("12345".to_string());
        let jira = AsyncJira::new(url, credentials).unwrap();
        jira.get::<EmptyResponse>("api", "/endpoint").await.unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_get_user() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/endpoint")
            .with_status(201)
            .match_header("authorization", "Basic dXNlcjpwd2Q=")
            .create_async()
            .await;

        // Run the test
        let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
        let jira = AsyncJira::new(url, credentials).unwrap();
        jira.get::<EmptyResponse>("api", "/endpoint").await.unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_get_cookie() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/endpoint")
            .with_status(201)
            .match_header("cookie", "JSESSIONID=ABC123XYZ")
            .create_async()
            .await;

        // Run the test
        let credentials = Credentials::Cookie("ABC123XYZ".to_string());
        let jira = AsyncJira::new(url, credentials).unwrap();
        jira.get::<EmptyResponse>("api", "/endpoint").await.unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_get() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/endpoint")
            .with_status(201)
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        jira.get::<EmptyResponse>("api", "/endpoint").await.unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_post() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("POST", "/rest/api/latest/endpoint")
            .with_status(201)
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let body = EmptyBody;
        jira.post::<EmptyResponse, EmptyBody>("api", "/endpoint", body)
            .await
            .unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_jira_http_put() {
        // Use async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Use async version of create
        let mock = server
            .mock("PUT", "/rest/api/latest/endpoint")
            .with_status(201)
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let body = EmptyBody;
        jira.put::<EmptyResponse, EmptyBody>("api", "/endpoint", body)
            .await
            .unwrap();

        // Use async version of assert
        mock.assert_async().await;
    }

    // Testing conversion without using tokio runtime
    #[test]
    fn test_convert_async_to_sync() {
        let async_jira = AsyncJira::new(JIRA_HOST, Credentials::Anonymous).unwrap();
        let _sync_jira = sync::Jira::from(&async_jira);

        // Just verify that the conversion completes without errors
        // This test passes if it compiles - we can't check internal fields since they're private
    }

    #[test]
    fn test_convert_sync_to_async() {
        let sync_jira = sync::Jira::new(JIRA_HOST, Credentials::Anonymous).unwrap();
        let _async_jira = sync_jira.into_async();

        // Just verify that the conversion completes without errors
        // This test passes if it compiles - we can't check internal fields since they're private
    }
}
