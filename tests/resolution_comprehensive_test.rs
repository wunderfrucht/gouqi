use gouqi::{Credentials, Jira, resolution::*};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn test_resolution_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let resolution = jira.resolution();

    // Just testing that we can create the resolution interface
    assert!(std::mem::size_of_val(&resolution) > 0);
}

#[test]
fn test_resolution_get_success() {
    let mut server = mockito::Server::new();

    let mut properties = BTreeMap::new();
    properties.insert("color".to_string(), json!("green"));
    properties.insert("level".to_string(), json!(1));

    let mock_resolution = json!({
        "id": "1",
        "title": "Fixed",
        "type": "resolution",
        "properties": properties,
        "additionalProperties": true
    });

    server
        .mock("GET", "/rest/api/latest/resolution/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_resolution.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.resolution().get("1");

    assert!(result.is_ok());
    let resolution = result.unwrap();
    assert_eq!(resolution.id, "1");
    assert_eq!(resolution.title, "Fixed");
    assert_eq!(resolution.resolution_type, "resolution");
    assert!(resolution.additional_properties);
    assert_eq!(resolution.properties.len(), 2);
    assert_eq!(resolution.properties.get("color").unwrap(), &json!("green"));
    assert_eq!(resolution.properties.get("level").unwrap(), &json!(1));
}

#[test]
fn test_resolution_get_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/resolution/999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Resolution does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.resolution().get("999");

    assert!(result.is_err());
}

#[test]
fn test_resolution_get_string_id() {
    let mut server = mockito::Server::new();

    let mock_resolution = json!({
        "id": "FIXED",
        "title": "Fixed",
        "type": "resolution",
        "properties": {},
        "additionalProperties": false
    });

    server
        .mock("GET", "/rest/api/latest/resolution/FIXED")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_resolution.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.resolution().get("FIXED");

    assert!(result.is_ok());
    let resolution = result.unwrap();
    assert_eq!(resolution.id, "FIXED");
    assert_eq!(resolution.title, "Fixed");
    assert_eq!(resolution.resolution_type, "resolution");
    assert!(!resolution.additional_properties);
    assert!(resolution.properties.is_empty());
}

#[test]
fn test_resolution_get_with_numeric_id_conversion() {
    let mut server = mockito::Server::new();

    let mock_resolution = json!({
        "id": "42",
        "title": "Won't Fix",
        "type": "resolution",
        "properties": {
            "reason": "out_of_scope"
        },
        "additionalProperties": true
    });

    server
        .mock("GET", "/rest/api/latest/resolution/42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_resolution.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test that Into<String> works for numeric IDs
    let result = jira.resolution().get(42.to_string());

    assert!(result.is_ok());
    let resolution = result.unwrap();
    assert_eq!(resolution.id, "42");
    assert_eq!(resolution.title, "Won't Fix");
    assert_eq!(
        resolution.properties.get("reason").unwrap(),
        &json!("out_of_scope")
    );
}

#[test]
fn test_resolution_deserialize_minimal() {
    // Test resolution with minimal required fields
    let json_data = json!({
        "id": "1",
        "title": "Minimal Resolution",
        "type": "resolution",
        "properties": {},
        "additionalProperties": false
    });

    let resolution: Resolved = serde_json::from_value(json_data).unwrap();
    assert_eq!(resolution.id, "1");
    assert_eq!(resolution.title, "Minimal Resolution");
    assert_eq!(resolution.resolution_type, "resolution");
    assert!(!resolution.additional_properties);
    assert!(resolution.properties.is_empty());
}

#[test]
fn test_resolution_deserialize_with_complex_properties() {
    // Test resolution with complex property values
    let json_data = json!({
        "id": "complex",
        "title": "Complex Resolution",
        "type": "custom_resolution",
        "properties": {
            "nested_object": {
                "key": "value",
                "number": 123
            },
            "array_value": [1, 2, 3],
            "boolean_value": true,
            "null_value": null,
            "string_value": "test"
        },
        "additionalProperties": true
    });

    let resolution: Resolved = serde_json::from_value(json_data).unwrap();
    assert_eq!(resolution.id, "complex");
    assert_eq!(resolution.title, "Complex Resolution");
    assert_eq!(resolution.resolution_type, "custom_resolution");
    assert!(resolution.additional_properties);
    assert_eq!(resolution.properties.len(), 5);

    // Check nested object
    let nested = resolution.properties.get("nested_object").unwrap();
    assert!(nested.is_object());

    // Check array
    let array = resolution.properties.get("array_value").unwrap();
    assert!(array.is_array());
    assert_eq!(array.as_array().unwrap().len(), 3);

    // Check boolean
    let boolean = resolution.properties.get("boolean_value").unwrap();
    assert!(boolean.is_boolean());
    assert!(boolean.as_bool().unwrap());

    // Check null
    let null_val = resolution.properties.get("null_value").unwrap();
    assert!(null_val.is_null());

    // Check string
    let string_val = resolution.properties.get("string_value").unwrap();
    assert!(string_val.is_string());
    assert_eq!(string_val.as_str().unwrap(), "test");
}

#[test]
fn test_resolution_interface_multiple_calls() {
    let mut server = mockito::Server::new();

    // Mock first resolution
    let mock_resolution1 = json!({
        "id": "1",
        "title": "Fixed",
        "type": "resolution",
        "properties": {},
        "additionalProperties": false
    });

    server
        .mock("GET", "/rest/api/latest/resolution/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_resolution1.to_string())
        .create();

    // Mock second resolution
    let mock_resolution2 = json!({
        "id": "2",
        "title": "Won't Fix",
        "type": "resolution",
        "properties": {},
        "additionalProperties": false
    });

    server
        .mock("GET", "/rest/api/latest/resolution/2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_resolution2.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let resolution_interface = jira.resolution();

    // Test multiple calls with the same interface
    let resolution1 = resolution_interface.get("1").unwrap();
    assert_eq!(resolution1.id, "1");
    assert_eq!(resolution1.title, "Fixed");

    let resolution2 = resolution_interface.get("2").unwrap();
    assert_eq!(resolution2.id, "2");
    assert_eq!(resolution2.title, "Won't Fix");
}

#[test]
fn test_resolution_server_error() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/resolution/error")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Internal server error"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.resolution().get("error");

    assert!(result.is_err());
}

#[test]
fn test_resolution_unauthorized() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/resolution/restricted")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Unauthorized"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.resolution().get("restricted");

    assert!(result.is_err());
}
