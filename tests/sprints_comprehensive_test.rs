use gouqi::{Credentials, Jira, SearchOptions, boards::Board, sprints::*};
use serde_json::json;

fn create_test_board() -> Board {
    Board {
        self_link: "http://localhost/rest/agile/latest/board/1".to_string(),
        id: 1,
        name: "Test Board".to_string(),
        type_name: "scrum".to_string(),
        location: None,
    }
}

fn create_test_board_with_id(id: u64) -> Board {
    Board {
        self_link: format!("http://localhost/rest/agile/latest/board/{}", id),
        id,
        name: format!("Test Board {}", id),
        type_name: "scrum".to_string(),
        location: None,
    }
}

#[test]
fn test_sprints_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let sprints = jira.sprints();

    // Just testing that we can create the sprints interface
    assert!(std::mem::size_of_val(&sprints) > 0);
}

#[test]
fn test_sprint_create_success() {
    let mut server = mockito::Server::new();

    let mock_sprint = json!({
        "id": 1,
        "self": format!("{}/rest/agile/latest/sprint/1", server.url()),
        "name": "New Sprint",
        "state": "active",
        "originBoardId": 1,
        "startDate": "2023-01-01T00:00:00.000Z",
        "endDate": "2023-01-14T23:59:59.999Z"
    });

    server
        .mock("POST", "/rest/agile/latest/sprint")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_sprint.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board();
    let result = jira.sprints().create(board, "New Sprint");

    assert!(result.is_ok());
    let sprint = result.unwrap();
    assert_eq!(sprint.id, 1);
    assert_eq!(sprint.name, "New Sprint");
    assert_eq!(sprint.state, Some("active".to_string()));
    assert_eq!(sprint.origin_board_id, Some(1));
    assert!(sprint.start_date.is_some());
    assert!(sprint.end_date.is_some());
}

#[test]
fn test_sprint_get_success() {
    let mut server = mockito::Server::new();

    let mock_sprint = json!({
        "id": 42,
        "self": format!("{}/rest/agile/latest/sprint/42", server.url()),
        "name": "Test Sprint",
        "state": "closed",
        "originBoardId": 1,
        "startDate": "2023-01-01T00:00:00.000Z",
        "endDate": "2023-01-14T23:59:59.999Z",
        "completeDate": "2023-01-14T18:00:00.000Z"
    });

    server
        .mock("GET", "/rest/agile/latest/sprint/42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprint.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.sprints().get("42");

    assert!(result.is_ok());
    let sprint = result.unwrap();
    assert_eq!(sprint.id, 42);
    assert_eq!(sprint.name, "Test Sprint");
    assert_eq!(sprint.state, Some("closed".to_string()));
    assert_eq!(sprint.origin_board_id, Some(1));
    assert!(sprint.start_date.is_some());
    assert!(sprint.end_date.is_some());
    assert!(sprint.complete_date.is_some());
}

#[test]
fn test_sprint_get_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/agile/latest/sprint/999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Sprint does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.sprints().get("999");

    assert!(result.is_err());
}

#[test]
fn test_sprint_move_issues_success() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/agile/latest/sprint/1/issue")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issues = vec!["PROJ-1".to_string(), "PROJ-2".to_string()];
    let result = jira.sprints().move_issues(1, issues);

    assert!(result.is_ok());
}

#[test]
fn test_sprint_move_issues_failure() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/agile/latest/sprint/1/issue")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Invalid issue keys"], "errors": {}}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let issues = vec!["INVALID-1".to_string()];
    let result = jira.sprints().move_issues(1, issues);

    assert!(result.is_err());
}

#[test]
fn test_sprint_list_success() {
    let mut server = mockito::Server::new();

    let mock_sprints = json!({
        "maxResults": 50,
        "startAt": 0,
        "isLast": true,
        "values": [
            {
                "id": 1,
                "self": format!("{}/rest/agile/latest/sprint/1", server.url()),
                "name": "Sprint 1",
                "state": "active",
                "originBoardId": 999
            },
            {
                "id": 2,
                "self": format!("{}/rest/agile/latest/sprint/2", server.url()),
                "name": "Sprint 2",
                "state": "closed",
                "originBoardId": 999,
                "startDate": "2023-01-01T00:00:00.000Z",
                "endDate": "2023-01-14T23:59:59.999Z",
                "completeDate": "2023-01-14T18:00:00.000Z"
            }
        ]
    });

    server
        .mock("GET", "/rest/agile/latest/board/1/sprint?")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprints.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board();
    let options = SearchOptions::default();
    let result = jira.sprints().list(&board, &options);

    assert!(result.is_ok());
    let sprint_results = result.unwrap();
    assert_eq!(sprint_results.max_results, 50);
    assert_eq!(sprint_results.start_at, 0);
    assert!(sprint_results.is_last);
    assert_eq!(sprint_results.values.len(), 2);

    assert_eq!(sprint_results.values[0].id, 1);
    assert_eq!(sprint_results.values[0].name, "Sprint 1");
    assert_eq!(sprint_results.values[0].state, Some("active".to_string()));

    assert_eq!(sprint_results.values[1].id, 2);
    assert_eq!(sprint_results.values[1].name, "Sprint 2");
    assert_eq!(sprint_results.values[1].state, Some("closed".to_string()));
    assert!(sprint_results.values[1].start_date.is_some());
    assert!(sprint_results.values[1].end_date.is_some());
    assert!(sprint_results.values[1].complete_date.is_some());
}

#[test]
fn test_sprint_list_with_search_options() {
    let mut server = mockito::Server::new();

    let mock_sprints = json!({
        "maxResults": 10,
        "startAt": 5,
        "isLast": false,
        "values": [
            {
                "id": 3,
                "self": format!("{}/rest/agile/latest/sprint/3", server.url()),
                "name": "Sprint 3",
                "state": "future",
                "originBoardId": 10
            }
        ]
    });

    server
        .mock(
            "GET",
            "/rest/agile/latest/board/10/sprint?maxResults=10&startAt=5",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprints.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board_with_id(10);
    let options = SearchOptions::default()
        .as_builder()
        .max_results(10)
        .start_at(5)
        .build();

    let result = jira.sprints().list(&board, &options);

    assert!(result.is_ok());
    let sprint_results = result.unwrap();
    assert_eq!(sprint_results.max_results, 10);
    assert_eq!(sprint_results.start_at, 5);
    assert!(!sprint_results.is_last);
    assert_eq!(sprint_results.values.len(), 1);
    assert_eq!(sprint_results.values[0].id, 3);
}

#[test]
fn test_sprint_iterator_single_page() {
    let mut server = mockito::Server::new();

    let mock_sprints = json!({
        "maxResults": 50,
        "startAt": 0,
        "isLast": true,
        "values": [
            {
                "id": 1,
                "self": format!("{}/rest/agile/latest/sprint/1", server.url()),
                "name": "Sprint 1",
                "state": "active",
                "originBoardId": 999
            },
            {
                "id": 2,
                "self": format!("{}/rest/agile/latest/sprint/2", server.url()),
                "name": "Sprint 2",
                "state": "closed",
                "originBoardId": 999
            }
        ]
    });

    server
        .mock("GET", "/rest/agile/latest/board/1/sprint?")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprints.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board();
    let options = SearchOptions::default();
    let mut iter = jira.sprints().iter(&board, &options).unwrap();

    // Iterator should return sprints in reverse order (pop from end)
    let sprint1 = iter.next();
    assert!(sprint1.is_some());
    let sprint1 = sprint1.unwrap();
    assert_eq!(sprint1.id, 2);
    assert_eq!(sprint1.name, "Sprint 2");

    let sprint2 = iter.next();
    assert!(sprint2.is_some());
    let sprint2 = sprint2.unwrap();
    assert_eq!(sprint2.id, 1);
    assert_eq!(sprint2.name, "Sprint 1");

    // No more items
    assert!(iter.next().is_none());
}

#[test]
#[ignore = "Intermittent test isolation issue with mockito"]
fn test_sprint_iterator_multiple_pages() {
    let mut server = mockito::Server::new();

    // First page
    let first_page = json!({
        "maxResults": 1,
        "startAt": 0,
        "isLast": false,
        "values": [
            {
                "id": 1,
                "self": format!("{}/rest/agile/latest/sprint/1", server.url()),
                "name": "Sprint 1",
                "state": "active",
                "originBoardId": 20
            }
        ]
    });

    // Second page
    let second_page = json!({
        "maxResults": 1,
        "startAt": 1,
        "isLast": true,
        "values": [
            {
                "id": 2,
                "self": format!("{}/rest/agile/latest/sprint/2", server.url()),
                "name": "Sprint 2",
                "state": "closed",
                "originBoardId": 20
            }
        ]
    });

    server
        .mock("GET", "/rest/agile/latest/board/20/sprint?maxResults=1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(first_page.to_string())
        .create();

    server
        .mock(
            "GET",
            "/rest/agile/latest/board/20/sprint?maxResults=1&startAt=1",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(second_page.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board_with_id(20);
    let options = SearchOptions::default().as_builder().max_results(1).build();

    let mut iter = jira.sprints().iter(&board, &options).unwrap();

    // First sprint from first page
    let sprint1 = iter.next();
    assert!(sprint1.is_some());
    let sprint1 = sprint1.unwrap();
    assert_eq!(sprint1.id, 1);

    // First (and only) sprint from second page
    let sprint2 = iter.next();
    assert!(sprint2.is_some());
    let sprint2 = sprint2.unwrap();
    assert_eq!(sprint2.id, 2);

    // No more items
    assert!(iter.next().is_none());
}

#[test]
fn test_sprint_iterator_error_handling() {
    let mut server = mockito::Server::new();

    // First successful request
    let first_page = json!({
        "maxResults": 1,
        "startAt": 0,
        "isLast": false,
        "values": [
            {
                "id": 1,
                "self": format!("{}/rest/agile/latest/sprint/1", server.url()),
                "name": "Sprint 1",
                "state": "active",
                "originBoardId": 999
            }
        ]
    });

    server
        .mock("GET", "/rest/agile/latest/board/30/sprint?maxResults=1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(first_page.to_string())
        .create();

    // Second request fails
    server
        .mock(
            "GET",
            "/rest/agile/latest/board/30/sprint?maxResults=1&startAt=1",
        )
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Internal server error"], "errors": {}}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = create_test_board_with_id(30);
    let options = SearchOptions::default().as_builder().max_results(1).build();

    let mut iter = jira.sprints().iter(&board, &options).unwrap();

    // First sprint should work
    let sprint1 = iter.next();
    assert!(sprint1.is_some());
    assert_eq!(sprint1.unwrap().id, 1);

    // Second call should fail gracefully and return None
    let sprint2 = iter.next();
    assert!(sprint2.is_none());
}

#[test]
fn test_sprint_deserialize_minimal() {
    // Test sprint with minimal required fields
    let json_data = json!({
        "id": 1,
        "self": "http://localhost/rest/agile/latest/sprint/1",
        "name": "Minimal Sprint"
    });

    let sprint: Sprint = serde_json::from_value(json_data).unwrap();
    assert_eq!(sprint.id, 1);
    assert_eq!(sprint.name, "Minimal Sprint");
    assert!(sprint.state.is_none());
    assert!(sprint.start_date.is_none());
    assert!(sprint.end_date.is_none());
    assert!(sprint.complete_date.is_none());
    assert!(sprint.origin_board_id.is_none());
}

#[test]
fn test_sprint_deserialize_with_dates() {
    // Test sprint with all date fields
    let json_data = json!({
        "id": 1,
        "self": "http://localhost/rest/agile/latest/sprint/1",
        "name": "Complete Sprint",
        "state": "closed",
        "startDate": "2023-01-01T00:00:00.000Z",
        "endDate": "2023-01-14T23:59:59.999Z",
        "completeDate": "2023-01-14T18:00:00.000Z",
        "originBoardId": 42
    });

    let sprint: Sprint = serde_json::from_value(json_data).unwrap();
    assert_eq!(sprint.id, 1);
    assert_eq!(sprint.name, "Complete Sprint");
    assert_eq!(sprint.state, Some("closed".to_string()));
    assert_eq!(sprint.origin_board_id, Some(42));

    // Verify dates are parsed correctly
    assert!(sprint.start_date.is_some());
    assert!(sprint.end_date.is_some());
    assert!(sprint.complete_date.is_some());

    let start_date = sprint.start_date.unwrap();
    assert_eq!(start_date.year(), 2023);
    assert_eq!(start_date.month() as u8, 1);
    assert_eq!(start_date.day(), 1);
}

#[test]
fn test_sprint_results_deserialize() {
    let json_data = json!({
        "maxResults": 25,
        "startAt": 10,
        "isLast": false,
        "values": []
    });

    let results: SprintResults = serde_json::from_value(json_data).unwrap();
    assert_eq!(results.max_results, 25);
    assert_eq!(results.start_at, 10);
    assert!(!results.is_last);
    assert!(results.values.is_empty());
}
