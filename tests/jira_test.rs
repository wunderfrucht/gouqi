extern crate gouqi;
extern crate mockito;
extern crate serde_json;

use gouqi::*;
use serde::Serialize;

const JIRA_HOST: &str = "http://jira.com";

#[derive(Serialize, Debug, Default)]
struct EmptyBody;

#[test]
fn jira_new_should_err_if_no_uri() {
    let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
    let jira = Jira::new("12345", credentials);
    assert!(jira.is_err());
}

#[test]
fn jira_new_should_ok_with_uri() {
    let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
    let jira = Jira::new(JIRA_HOST, credentials);
    assert!(jira.is_ok());
}

#[test]
fn jira_http_delete() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("DELETE", "/rest/api/latest/endpoint")
        .with_status(201)
        .create();

    let jira = Jira::new(url, Credentials::Anonymous).unwrap();
    jira.delete::<EmptyResponse>("api", "/endpoint").unwrap();
    mock.assert();
}

#[test]
fn jira_http_get_bearer() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("GET", "/rest/api/latest/endpoint")
        .with_status(201)
        .match_header("authorization", "Bearer 12345")
        .create();
    let credentials = Credentials::Bearer("12345".to_string());

    let jira = Jira::new(url, credentials).unwrap();
    jira.get::<EmptyResponse>("api", "/endpoint").unwrap();
    mock.assert();
}

#[test]
fn jira_http_get_user() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("GET", "/rest/api/latest/endpoint")
        .with_status(201)
        .match_header("authorization", "Basic dXNlcjpwd2Q=")
        .create();
    let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());

    let jira = Jira::new(url, credentials).unwrap();
    jira.get::<EmptyResponse>("api", "/endpoint").unwrap();
    mock.assert();
}

#[test]
fn jira_http_get_cookie() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("GET", "/rest/api/latest/endpoint")
        .with_status(201)
        .match_header("cookie", "JSESSIONID=ABC123XYZ")
        .create();
    let credentials = Credentials::Cookie("ABC123XYZ".to_string());

    let jira = Jira::new(url, credentials).unwrap();
    jira.get::<EmptyResponse>("api", "/endpoint").unwrap();
    mock.assert();
}

#[test]
fn jira_http_get() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("GET", "/rest/api/latest/endpoint")
        .with_status(201)
        .create();

    let jira = Jira::new(url, Credentials::Anonymous).unwrap();
    jira.get::<EmptyResponse>("api", "/endpoint").unwrap();
    mock.assert();
}

#[test]
fn jira_http_post() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("POST", "/rest/api/latest/endpoint")
        .with_status(201)
        .create();

    let jira = Jira::new(url, Credentials::Anonymous).unwrap();
    let body = EmptyBody;
    jira.post::<EmptyResponse, EmptyBody>("api", "/endpoint", body)
        .unwrap();
    mock.assert();
}

#[test]
fn jira_http_put() {
    let mut server = mockito::Server::new();

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    let mock = server
        .mock("PUT", "/rest/api/latest/endpoint")
        .with_status(201)
        .create();

    let jira = Jira::new(url, Credentials::Anonymous).unwrap();
    let body = EmptyBody;
    jira.put::<EmptyResponse, EmptyBody>("api", "/endpoint", body)
        .unwrap();
    mock.assert();
}
