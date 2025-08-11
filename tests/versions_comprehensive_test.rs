use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_versions_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let versions = jira.versions();
    
    // Just testing that we can create the versions interface
    assert!(std::mem::size_of_val(&versions) > 0);
}

#[test]
fn test_project_versions_success() {
    let mut server = mockito::Server::new();
    
    let mock_versions = json!([
        {
            "id": "10000",
            "name": "Version 1.0",
            "archived": false,
            "released": true,
            "projectId": 10001,
            "self": format!("{}/rest/api/2/version/10000", server.url())
        },
        {
            "id": "10001", 
            "name": "Version 2.0",
            "archived": false,
            "released": false,
            "projectId": 10001,
            "self": format!("{}/rest/api/2/version/10001", server.url())
        }
    ]);

    server
        .mock("GET", "/rest/api/2/project/TEST/versions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_versions.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().project_versions("TEST");

    assert!(result.is_ok());
    let versions = result.unwrap();
    assert_eq!(versions.len(), 2);
    
    assert_eq!(versions[0].id, "10000");
    assert_eq!(versions[0].name, "Version 1.0");
    assert!(!versions[0].archived);
    assert!(versions[0].released);
    assert_eq!(versions[0].project_id, 10001);
    
    assert_eq!(versions[1].id, "10001");
    assert_eq!(versions[1].name, "Version 2.0");
    assert!(!versions[1].archived);
    assert!(!versions[1].released);
    assert_eq!(versions[1].project_id, 10001);
}

#[test]
fn test_project_versions_empty() {
    let mut server = mockito::Server::new();
    
    server
        .mock("GET", "/rest/api/2/project/EMPTY/versions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().project_versions("EMPTY");

    assert!(result.is_ok());
    let versions = result.unwrap();
    assert!(versions.is_empty());
}

#[test]
fn test_project_versions_not_found() {
    let mut server = mockito::Server::new();
    
    server
        .mock("GET", "/rest/api/2/project/NOTFOUND/versions")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Project not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().project_versions("NOTFOUND");

    assert!(result.is_err());
}

#[test]
fn test_create_version_success() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10002",
        "name": "New Version",
        "archived": false,
        "released": false,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10002", server.url())
    });

    server
        .mock("POST", "/rest/api/2/version")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().create(10001, "New Version");

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version.id, "10002");
    assert_eq!(version.name, "New Version");
    assert!(!version.archived);
    assert!(!version.released);
    assert_eq!(version.project_id, 10001);
    assert!(version.self_link.contains("/version/10002"));
}

#[test]
fn test_create_version_with_string() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10003",
        "name": "Version from String",
        "archived": false,
        "released": false,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10003", server.url())
    });

    server
        .mock("POST", "/rest/api/2/version")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version_name = String::from("Version from String");
    let result = jira.versions().create(10001, version_name);

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version.name, "Version from String");
}

#[test]
fn test_create_version_invalid_project() {
    let mut server = mockito::Server::new();
    
    server
        .mock("POST", "/rest/api/2/version")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Invalid project ID"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().create(99999, "Invalid Version");

    assert!(result.is_err());
}

#[test]
fn test_move_after_success() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10000",
        "name": "Moved Version",
        "archived": false,
        "released": false,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10000", server.url())
    });

    server
        .mock("POST", "/rest/api/2/version/10000/move")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Test Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let result = jira.versions().move_after(&version, "after-version-id");

    assert!(result.is_ok());
    let moved_version = result.unwrap();
    assert_eq!(moved_version.id, "10000");
    assert_eq!(moved_version.name, "Moved Version");
}

#[test]
fn test_move_after_with_string() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10000",
        "name": "Moved Version",
        "archived": false,
        "released": false,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10000", server.url())
    });

    server
        .mock("POST", "/rest/api/2/version/10000/move")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Test Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let after_id = String::from("string-after-id");
    let result = jira.versions().move_after(&version, after_id);

    assert!(result.is_ok());
}

#[test]
fn test_move_after_version_not_found() {
    let mut server = mockito::Server::new();
    
    server
        .mock("POST", "/rest/api/2/version/99999/move")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Version not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "99999".to_string(),
        name: "Nonexistent Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/99999".to_string(),
    };
    
    let result = jira.versions().move_after(&version, "after-id");

    assert!(result.is_err());
}

#[test]
fn test_release_version_success() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10000",
        "name": "Released Version",
        "archived": false,
        "released": true,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10000", server.url())
    });

    server
        .mock("PUT", "/rest/api/2/version/10000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Test Version".to_string(),
        archived: false,
        released: false, // Not yet released
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let result = jira.versions().release(&version, None);

    assert!(result.is_ok());
}

#[test]
fn test_release_version_with_move_unfixed_issues() {
    let mut server = mockito::Server::new();
    
    let mock_version = json!({
        "id": "10000",
        "name": "Released Version",
        "archived": false,
        "released": true,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10000", server.url())
    });

    server
        .mock("PUT", "/rest/api/2/version/10000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Test Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let target_version = gouqi::Version {
        id: "10001".to_string(),
        name: "Target Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/10001".to_string(),
    };
    
    let result = jira.versions().release(&version, Some(&target_version));

    assert!(result.is_ok());
}

#[test]
fn test_release_already_released_version() {
    // No server mock needed - this should return early without API call
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Already Released".to_string(),
        archived: false,
        released: true, // Already released
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let result = jira.versions().release(&version, None);

    // Should succeed without making API call
    assert!(result.is_ok());
}

#[test]
fn test_release_version_server_error() {
    let mut server = mockito::Server::new();
    
    server
        .mock("PUT", "/rest/api/2/version/10000")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Internal server error"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let version = gouqi::Version {
        id: "10000".to_string(),
        name: "Error Version".to_string(),
        archived: false,
        released: false,
        project_id: 10001,
        self_link: "http://test.com/version/10000".to_string(),
    };
    
    let result = jira.versions().release(&version, None);

    assert!(result.is_err());
}

#[test]
fn test_version_serialization_deserialization() {
    // Test Version struct serialization/deserialization
    let json_data = json!({
        "id": "12345",
        "name": "Test Version",
        "archived": true,
        "released": false,
        "projectId": 54321,
        "self": "http://test.com/rest/api/2/version/12345"
    });

    let version: gouqi::Version = serde_json::from_value(json_data.clone()).unwrap();
    assert_eq!(version.id, "12345");
    assert_eq!(version.name, "Test Version");
    assert!(version.archived);
    assert!(!version.released);
    assert_eq!(version.project_id, 54321);
    assert_eq!(version.self_link, "http://test.com/rest/api/2/version/12345");
    
    // Test serialization back to JSON
    let serialized = serde_json::to_value(&version).unwrap();
    assert_eq!(serialized["id"], "12345");
    assert_eq!(serialized["name"], "Test Version");
    assert_eq!(serialized["archived"], true);
    assert_eq!(serialized["released"], false);
    assert_eq!(serialized["projectId"], 54321);
    assert_eq!(serialized["self"], "http://test.com/rest/api/2/version/12345");
}

#[test]
fn test_version_creation_body_serialization() {
    let creation_body = gouqi::VersionCreationBody {
        name: "New Test Version".to_string(),
        project_id: 12345,
    };

    let json = serde_json::to_string(&creation_body).unwrap();
    assert!(json.contains("New Test Version"));
    assert!(json.contains("projectId"));
    assert!(json.contains("12345"));
    
    // Verify correct field naming
    let json_value: serde_json::Value = serde_json::to_value(&creation_body).unwrap();
    assert_eq!(json_value["name"], "New Test Version");
    assert_eq!(json_value["projectId"], 12345);
}

#[test]
fn test_version_move_after_body_serialization() {
    let move_body = gouqi::VersionMoveAfterBody {
        after: "target-version-id".to_string(),
    };

    let json = serde_json::to_string(&move_body).unwrap();
    assert!(json.contains("target-version-id"));
    assert!(json.contains("after"));
}

#[test]
fn test_version_update_body_serialization() {
    let update_body = gouqi::VersionUpdateBody {
        released: true,
        archived: false,
        move_unfixed_issues_to: Some("target-version-self-link".to_string()),
    };

    let json_value: serde_json::Value = serde_json::to_value(&update_body).unwrap();
    assert_eq!(json_value["released"], true);
    assert_eq!(json_value["archived"], false);
    assert_eq!(json_value["moveUnfixedIssuesTo"], "target-version-self-link");
    
    // Test with None value
    let update_body_none = gouqi::VersionUpdateBody {
        released: false,
        archived: true,
        move_unfixed_issues_to: None,
    };

    let json_value_none: serde_json::Value = serde_json::to_value(&update_body_none).unwrap();
    assert_eq!(json_value_none["released"], false);
    assert_eq!(json_value_none["archived"], true);
    assert!(json_value_none["moveUnfixedIssuesTo"].is_null());
}

#[test]
fn test_versions_interface_multiple_operations() {
    let mut server = mockito::Server::new();
    
    // Mock project versions
    let mock_project_versions = json!([
        {
            "id": "10000",
            "name": "Version 1.0",
            "archived": false,
            "released": true,
            "projectId": 10001,
            "self": format!("{}/rest/api/2/version/10000", server.url())
        }
    ]);

    server
        .mock("GET", "/rest/api/2/project/MULTI/versions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_project_versions.to_string())
        .create();

    // Mock create version
    let mock_created_version = json!({
        "id": "10002",
        "name": "Multi Test Version",
        "archived": false,
        "released": false,
        "projectId": 10001,
        "self": format!("{}/rest/api/2/version/10002", server.url())
    });

    server
        .mock("POST", "/rest/api/2/version")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_created_version.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let versions_interface = jira.versions();

    // Test multiple operations with the same interface
    let project_versions = versions_interface.project_versions("MULTI").unwrap();
    assert_eq!(project_versions.len(), 1);
    assert_eq!(project_versions[0].name, "Version 1.0");
    
    let new_version = versions_interface.create(10001, "Multi Test Version").unwrap();
    assert_eq!(new_version.id, "10002");
    assert_eq!(new_version.name, "Multi Test Version");
}