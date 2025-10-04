//! Tests for AsyncTransitions implementation

#[cfg(feature = "async")]
mod async_transitions_tests {
    use gouqi::{Credentials, r#async::Jira as AsyncJira};
    use serde_json::json;

    #[tokio::test]
    async fn test_async_transitions_list() {
        let mut server = mockito::Server::new_async().await;

        let mock_transitions = json!({
            "transitions": [
                {
                    "id": "11",
                    "name": "To Do",
                    "to": {
                        "self": format!("{}/rest/api/latest/status/10000", server.url()),
                        "description": "To Do status",
                        "name": "To Do",
                        "id": "10000"
                    }
                },
                {
                    "id": "21",
                    "name": "In Progress",
                    "to": {
                        "self": format!("{}/rest/api/latest/status/10001", server.url()),
                        "description": "In Progress status",
                        "name": "In Progress",
                        "id": "10001"
                    }
                }
            ]
        });

        server
            .mock(
                "GET",
                "/rest/api/latest/issue/TEST-1/transitions?expand=transitions.fields",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_transitions.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.transitions("TEST-1").list().await;

        assert!(result.is_ok());
        let transitions = result.unwrap();
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].name, "To Do");
    }

    #[tokio::test]
    async fn test_async_transitions_trigger() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/transitions")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let transition = gouqi::TransitionTriggerOptions::builder("11").build();
        let result = jira.transitions("TEST-1").trigger(transition).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_transitions_trigger_with_resolution() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/transitions")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let transition = gouqi::TransitionTriggerOptions::builder("11")
            .resolution("Done")
            .build();
        let result = jira.transitions("TEST-1").trigger(transition).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_transitions_not_found() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock(
                "GET",
                "/rest/api/latest/issue/INVALID-1/transitions?expand=transitions.fields",
            )
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Issue not found"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.transitions("INVALID-1").list().await;

        assert!(result.is_err());
    }
}
