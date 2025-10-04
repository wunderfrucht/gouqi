//! Comprehensive async tests for boards

#[cfg(feature = "async")]
mod async_boards_tests {
    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::{Credentials, SearchOptions};
    use serde_json::json;

    #[tokio::test]
    async fn test_async_board_get() {
        let mut server = mockito::Server::new_async().await;

        let mock_board = json!({
            "self": format!("{}/rest/agile/latest/board/1", server.url()),
            "id": 1,
            "name": "Test Board",
            "type": "scrum",
            "location": {
                "projectId": 10000,
                "displayName": "Test Project",
                "projectName": "Test Project",
                "projectKey": "TEST",
                "projectTypeKey": "software"
            }
        });

        server
            .mock("GET", "/rest/agile/latest/board/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_board.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.boards().get(1u64).await;

        assert!(result.is_ok());
        let board = result.unwrap();
        assert_eq!(board.id, 1);
        assert_eq!(board.name, "Test Board");
    }

    #[tokio::test]
    async fn test_async_board_list() {
        let mut server = mockito::Server::new_async().await;

        let mock_boards = json!({
            "maxResults": 50,
            "startAt": 0,
            "isLast": true,
            "values": [
                {
                    "self": format!("{}/rest/agile/latest/board/1", server.url()),
                    "id": 1,
                    "name": "Scrum Board",
                    "type": "scrum"
                },
                {
                    "self": format!("{}/rest/agile/latest/board/2", server.url()),
                    "id": 2,
                    "name": "Kanban Board",
                    "type": "kanban"
                }
            ]
        });

        server
            .mock("GET", "/rest/agile/latest/board?")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_boards.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = SearchOptions::default();
        let result = jira.boards().list(&options).await;

        assert!(result.is_ok());
        let board_results = result.unwrap();
        assert_eq!(board_results.values.len(), 2);
    }

    #[tokio::test]
    async fn test_async_board_get_not_found() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/agile/latest/board/999")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Board does not exist"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.boards().get(999u64).await;

        assert!(result.is_err());
    }
}
