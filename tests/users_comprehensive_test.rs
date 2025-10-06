use gouqi::{Credentials, Jira, User, users::*};
use serde_json::json;

#[test]
fn test_users_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let users = jira.users();

    // Just testing that we can create the users interface
    assert!(std::mem::size_of_val(&users) > 0);
}

#[test]
fn test_user_get_success() {
    let mut server = mockito::Server::new();

    let mock_user = json!({
        "accountId": "5b10a2844c20165700ede21g",
        "accountType": "atlassian",
        "displayName": "John Doe",
        "emailAddress": "john@example.com",
        "active": true,
        "avatarUrls": {
            "48x48": "https://avatar-management.services.atlassian.com/initials/JD-0.png"
        },
        "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
    });

    server
        .mock(
            "GET",
            "/rest/api/latest/user?accountId=5b10a2844c20165700ede21g",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_user.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result: Result<User, _> = jira.users().get("5b10a2844c20165700ede21g");

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.account_id.unwrap(), "5b10a2844c20165700ede21g");
    assert_eq!(user.display_name, "John Doe");
}

#[test]
fn test_user_get_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/user?accountId=invalid")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["User does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result: Result<User, _> = jira.users().get("invalid");

    assert!(result.is_err());
}

#[test]
fn test_user_search_success() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "accountType": "atlassian",
            "displayName": "John Doe",
            "emailAddress": "john@example.com",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        },
        {
            "accountId": "5b10a2844c20165700ede22h",
            "accountType": "atlassian",
            "displayName": "Jane Doe",
            "emailAddress": "jane@example.com",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede22h", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/search?query=john&maxResults=10",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = UserSearchOptions::builder()
        .query("john")
        .max_results(10)
        .build();

    let result: Result<Vec<User>, _> = jira.users().search(&options);

    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].display_name, "John Doe");
    assert_eq!(users[1].display_name, "Jane Doe");
}

#[test]
fn test_user_search_empty_query() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "displayName": "User 1",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        }
    ]);

    server
        .mock("GET", "/rest/api/latest/user/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = UserSearchOptions::default();
    let result: Result<Vec<User>, _> = jira.users().search(&options);

    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 1);
}

#[test]
fn test_user_search_with_pagination() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede23i",
            "displayName": "User 3",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede23i", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/search?startAt=10&maxResults=5",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = UserSearchOptions::builder()
        .start_at(10)
        .max_results(5)
        .build();

    let result: Result<Vec<User>, _> = jira.users().search(&options);

    assert!(result.is_ok());
}

#[test]
fn test_get_assignable_users_for_project() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "displayName": "Assignable User 1",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        },
        {
            "accountId": "5b10a2844c20165700ede22h",
            "displayName": "Assignable User 2",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede22h", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/assignable/search?maxResults=50&project=TEST",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = AssignableUserOptions::builder().max_results(50).build();

    let result: Result<Vec<User>, _> = jira.users().get_assignable_users("TEST", &options);

    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].display_name, "Assignable User 1");
}

#[test]
fn test_get_assignable_users_for_issue() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "displayName": "Assignable User 1",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/assignable/search?issueKey=TEST-123",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = AssignableUserOptions::default();
    let result: Result<Vec<User>, _> = jira
        .users()
        .get_assignable_users_for_issue("TEST-123", &options);

    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 1);
}

#[test]
fn test_assignable_user_options_builder() {
    let options = AssignableUserOptions::builder()
        .query("john")
        .start_at(5)
        .max_results(25)
        .build();

    assert_eq!(options.query, Some("john".to_string()));
    assert_eq!(options.start_at, Some(5));
    assert_eq!(options.max_results, Some(25));
}

#[test]
fn test_user_search_options_builder() {
    let options = UserSearchOptions::builder()
        .query("admin")
        .start_at(0)
        .max_results(50)
        .build();

    assert_eq!(options.query, Some("admin".to_string()));
    assert_eq!(options.start_at, Some(0));
    assert_eq!(options.max_results, Some(50));
}

#[test]
fn test_user_search_unauthorized() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/user/search?query=test")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Unauthorized"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = UserSearchOptions::builder().query("test").build();

    let result: Result<Vec<User>, _> = jira.users().search(&options);

    assert!(result.is_err());
}
