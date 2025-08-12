use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_transitions_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let transitions = jira.transitions("TEST-123");

    // Just testing that we can create the transitions interface
    assert!(std::mem::size_of_val(&transitions) > 0);
}

#[test]
fn test_transitions_list_success() {
    let mut server = mockito::Server::new();

    let mock_transitions = json!({
        "transitions": [
            {
                "id": "11",
                "name": "To Do",
                "to": {
                    "id": "10000",
                    "name": "To Do"
                }
            },
            {
                "id": "21",
                "name": "In Progress",
                "to": {
                    "id": "10001",
                    "name": "In Progress"
                }
            },
            {
                "id": "31",
                "name": "Done",
                "to": {
                    "id": "10002",
                    "name": "Done"
                }
            }
        ]
    });

    server
        .mock(
            "GET",
            "/rest/api/latest/issue/TEST-123/transitions?expand=transitions.fields",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_transitions.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.transitions("TEST-123").list();

    assert!(result.is_ok());
    let transitions = result.unwrap();
    assert_eq!(transitions.len(), 3);

    assert_eq!(transitions[0].id, "11");
    assert_eq!(transitions[0].name, "To Do");
    assert_eq!(transitions[0].to.id, "10000");
    assert_eq!(transitions[0].to.name, "To Do");

    assert_eq!(transitions[1].id, "21");
    assert_eq!(transitions[1].name, "In Progress");
    assert_eq!(transitions[1].to.id, "10001");
    assert_eq!(transitions[1].to.name, "In Progress");

    assert_eq!(transitions[2].id, "31");
    assert_eq!(transitions[2].name, "Done");
    assert_eq!(transitions[2].to.id, "10002");
    assert_eq!(transitions[2].to.name, "Done");
}

#[test]
fn test_transitions_list_empty() {
    let mut server = mockito::Server::new();

    let mock_transitions = json!({
        "transitions": []
    });

    server
        .mock(
            "GET",
            "/rest/api/latest/issue/EMPTY-1/transitions?expand=transitions.fields",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_transitions.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.transitions("EMPTY-1").list();

    assert!(result.is_ok());
    let transitions = result.unwrap();
    assert!(transitions.is_empty());
}

#[test]
fn test_transitions_list_issue_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock(
            "GET",
            "/rest/api/latest/issue/NOTFOUND-1/transitions?expand=transitions.fields",
        )
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Issue does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.transitions("NOTFOUND-1").list();

    assert!(result.is_err());
}

#[test]
fn test_transition_trigger_success() {
    let mut server = mockito::Server::new();

    // Transition API typically returns empty body on success
    server
        .mock("POST", "/rest/api/latest/issue/TEST-123/transitions")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let trigger_options = gouqi::TransitionTriggerOptions::new("31");

    let result = jira.transitions("TEST-123").trigger(trigger_options);

    assert!(result.is_ok());
}

#[test]
fn test_transition_trigger_with_builder() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-456/transitions")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let trigger_options = gouqi::TransitionTriggerOptions::builder("21")
        .resolution("Fixed")
        .build();

    let result = jira.transitions("TEST-456").trigger(trigger_options);

    assert!(result.is_ok());
}

#[test]
fn test_transition_trigger_with_custom_fields() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-789/transitions")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let mut trigger_options = gouqi::TransitionTriggerOptions::builder("11");
    trigger_options
        .field("assignee", json!({"name": "john.doe"}))
        .field("comment", "Transitioning to To Do")
        .resolution("Won't Fix");

    let result = jira
        .transitions("TEST-789")
        .trigger(trigger_options.build());

    assert!(result.is_ok());
}

#[test]
fn test_transition_trigger_invalid_transition() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-123/transitions")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Invalid transition"], "errors": {}}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let trigger_options = gouqi::TransitionTriggerOptions::new("999");

    let result = jira.transitions("TEST-123").trigger(trigger_options);

    assert!(result.is_err());
}

#[test]
fn test_transition_trigger_unauthorized() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/SECRET-1/transitions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Unauthorized"], "errors": {}}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let trigger_options = gouqi::TransitionTriggerOptions::new("11");

    let result = jira.transitions("SECRET-1").trigger(trigger_options);

    assert!(result.is_err());
}

#[test]
fn test_transition_trigger_handles_serde_error() {
    let mut server = mockito::Server::new();

    // Return malformed JSON that would cause serde error
    server
        .mock("POST", "/rest/api/latest/issue/TEST-SERDE/transitions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let trigger_options = gouqi::TransitionTriggerOptions::new("11");

    let result = jira.transitions("TEST-SERDE").trigger(trigger_options);

    // Should handle serde error gracefully and return Ok(())
    assert!(result.is_ok());
}

#[test]
fn test_transition_option_deserialization() {
    let json_data = json!({
        "id": "11",
        "name": "Start Progress",
        "to": {
            "id": "3",
            "name": "In Progress"
        }
    });

    let transition: gouqi::TransitionOption = serde_json::from_value(json_data).unwrap();
    assert_eq!(transition.id, "11");
    assert_eq!(transition.name, "Start Progress");
    assert_eq!(transition.to.id, "3");
    assert_eq!(transition.to.name, "In Progress");
}

#[test]
fn test_transition_options_deserialization() {
    let json_data = json!({
        "transitions": [
            {
                "id": "11",
                "name": "To Do",
                "to": {
                    "id": "1",
                    "name": "To Do"
                }
            },
            {
                "id": "21",
                "name": "Done",
                "to": {
                    "id": "3",
                    "name": "Done"
                }
            }
        ]
    });

    let options: gouqi::TransitionOptions = serde_json::from_value(json_data).unwrap();
    assert_eq!(options.transitions.len(), 2);
    assert_eq!(options.transitions[0].id, "11");
    assert_eq!(options.transitions[1].id, "21");
}

#[test]
fn test_transition_trigger_options_new() {
    let trigger_options = gouqi::TransitionTriggerOptions::new("21");

    assert_eq!(trigger_options.transition.id, "21");
    assert!(trigger_options.fields.is_empty());
}

#[test]
fn test_transition_trigger_options_builder_new() {
    let builder = gouqi::TransitionTriggerOptions::builder("31");

    assert_eq!(builder.transition.id, "31");
    assert!(builder.fields.is_empty());
}

#[test]
fn test_transition_trigger_options_builder_field() {
    let mut builder = gouqi::TransitionTriggerOptions::builder("11");
    builder.field("assignee", json!({"name": "test.user"}));
    builder.field("priority", json!({"name": "High"}));

    assert_eq!(builder.fields.len(), 2);
    assert!(builder.fields.contains_key("assignee"));
    assert!(builder.fields.contains_key("priority"));

    let assignee_value = &builder.fields["assignee"];
    assert_eq!(assignee_value["name"], "test.user");
}

#[test]
fn test_transition_trigger_options_builder_resolution() {
    let mut builder = gouqi::TransitionTriggerOptions::builder("31");
    builder.resolution("Fixed");

    assert_eq!(builder.fields.len(), 1);
    assert!(builder.fields.contains_key("resolution"));

    let resolution_value = &builder.fields["resolution"];
    assert_eq!(resolution_value["name"], "Fixed");
}

#[test]
fn test_transition_trigger_options_builder_build() {
    let mut builder = gouqi::TransitionTriggerOptions::builder("21");
    builder
        .field("comment", "Moving to progress")
        .resolution("Fixed");

    let trigger_options = builder.build();

    assert_eq!(trigger_options.transition.id, "21");
    assert_eq!(trigger_options.fields.len(), 2);
    assert!(trigger_options.fields.contains_key("comment"));
    assert!(trigger_options.fields.contains_key("resolution"));
}

#[test]
fn test_transition_trigger_options_serialization() {
    let trigger_options = gouqi::TransitionTriggerOptions::new("11");

    let json_value = serde_json::to_value(&trigger_options).unwrap();
    assert!(json_value.is_object());
    assert!(json_value["transition"].is_object());
    assert_eq!(json_value["transition"]["id"], "11");
    assert!(json_value["fields"].is_object());
}

#[test]
fn test_transition_trigger_options_with_fields_serialization() {
    let mut builder = gouqi::TransitionTriggerOptions::builder("31");
    builder
        .field("assignee", json!({"name": "john.doe"}))
        .resolution("Done");

    let trigger_options = builder.build();
    let json_value = serde_json::to_value(&trigger_options).unwrap();

    assert_eq!(json_value["transition"]["id"], "31");
    assert_eq!(json_value["fields"]["assignee"]["name"], "john.doe");
    assert_eq!(json_value["fields"]["resolution"]["name"], "Done");
}

#[test]
fn test_transition_serialization() {
    let transition = gouqi::Transition {
        id: "21".to_string(),
    };

    let json_value = serde_json::to_value(&transition).unwrap();
    assert_eq!(json_value["id"], "21");
}

#[test]
fn test_transitions_interface_with_string_key() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let key = String::from("PROJECT-456");
    let transitions = jira.transitions(key);

    // Test that Into<String> trait works properly
    assert!(std::mem::size_of_val(&transitions) > 0);
}

#[test]
fn test_transitions_interface_multiple_operations() {
    let mut server = mockito::Server::new();

    // Mock list transitions
    let mock_transitions = json!({
        "transitions": [
            {
                "id": "11",
                "name": "Start Progress",
                "to": {
                    "id": "3",
                    "name": "In Progress"
                }
            }
        ]
    });

    server
        .mock(
            "GET",
            "/rest/api/latest/issue/MULTI-1/transitions?expand=transitions.fields",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_transitions.to_string())
        .create();

    // Mock trigger transition
    server
        .mock("POST", "/rest/api/latest/issue/MULTI-1/transitions")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let transitions_interface = jira.transitions("MULTI-1");

    // Test multiple operations with the same interface
    let available_transitions = transitions_interface.list().unwrap();
    assert_eq!(available_transitions.len(), 1);
    assert_eq!(available_transitions[0].name, "Start Progress");

    let trigger_options = gouqi::TransitionTriggerOptions::new("11");
    let trigger_result = transitions_interface.trigger(trigger_options);
    assert!(trigger_result.is_ok());
}

#[test]
fn test_transitions_builder_chain() {
    // Test that the builder pattern works with method chaining
    let trigger_options = gouqi::TransitionTriggerOptions::builder("31")
        .field("assignee", json!({"name": "test.user"}))
        .field("priority", json!({"name": "High"}))
        .resolution("Fixed")
        .build();

    assert_eq!(trigger_options.transition.id, "31");
    assert_eq!(trigger_options.fields.len(), 3);
    assert!(trigger_options.fields.contains_key("assignee"));
    assert!(trigger_options.fields.contains_key("priority"));
    assert!(trigger_options.fields.contains_key("resolution"));
}
