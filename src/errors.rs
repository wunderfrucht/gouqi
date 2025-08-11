use reqwest::StatusCode;
use thiserror::Error;

// Ours
use crate::Errors;

/// An enumeration over potential errors that may
/// happen when sending a request to jira
#[derive(Error, Debug)]
pub enum Error {
    /// Error associated with http request
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// Error associated IO
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    /// Error associated with parsing or serializing
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Client request errors
    #[error("Jira client error ({code}):\n{errors:#?}")]
    Fault { code: StatusCode, errors: Errors },
    /// Invalid credentials
    #[error("Could not connect to Jira: Unauthorized")]
    Unauthorized,
    /// HTTP method is not allowed
    #[error("Jira request error: MethodNotAllowed")]
    MethodNotAllowed,
    /// Page not found
    #[error("Jira request error: NotFound")]
    NotFound,
    /// URI parse error
    #[error("Could not connect to Jira: {0}")]
    ParseError(#[from] url::ParseError),
}
