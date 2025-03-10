extern crate reqwest;
extern crate serde;
extern crate tracing;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

#[cfg(feature = "async")]
extern crate futures;
#[cfg(feature = "async")]
extern crate tokio;

// Public re-exports only
// No imports needed for the root module

#[cfg(feature = "async")]
pub mod r#async;
pub mod sync;

pub mod attachments;
mod builder;
pub mod components;
mod errors;
pub mod issues;
mod rep;
mod search;
mod transitions;
mod versions;

pub use crate::attachments::*;
pub use crate::builder::*;
pub use crate::components::*;
pub use crate::errors::*;
pub use crate::issues::*;
pub use crate::rep::*;
#[cfg(feature = "async")]
pub use crate::search::AsyncSearch;
pub use crate::search::Search;
pub use crate::transitions::*;
pub mod boards;
pub mod resolution;
pub use crate::boards::*;
pub mod sprints;
pub use crate::sprints::*;
pub use crate::versions::*;

// Re-export the synchronous API as the default for backward compatibility
pub use sync::Jira;

#[derive(Deserialize, Debug)]
pub struct EmptyResponse;

pub type Result<T> = std::result::Result<T, Error>;

/// Types of authentication credentials
///
/// # Notes
///
/// - Personal Access Token are used with [`Credentials::Basic`] scheme as a password replacement and *not* as a [`Credentials::Bearer`]
///   like the [API documentation sugest](https://developer.atlassian.com/server/jira/platform/rest-apis/#authentication-and-authorization).
#[derive(Clone, Debug)]
pub enum Credentials {
    /// Use no authentication
    Anonymous,
    /// Username and password credentials (Personal Access Token count as a password)
    Basic(String, String),
    /// Authentification via bearer token
    Bearer(String),
    // TODO: Add OAuth
}

impl Credentials {
    fn apply(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match self {
            Credentials::Anonymous => request,
            Credentials::Basic(ref user, ref pass) => {
                request.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(ref token) => request.bearer_auth(token.to_owned()),
        }
    }
}

#[cfg(feature = "async")]
impl crate::sync::Jira {
    /// Convert a synchronous client to an asynchronous one.
    /// Note that this requires the "async" feature to be enabled.
    pub fn into_async(&self) -> crate::r#async::Jira {
        // This is a best-effort conversion - if it fails, we'll just create a new client
        if let Ok(jira) = crate::r#async::Jira::new(self.host.as_str(), self.credentials.clone()) {
            jira
        } else {
            // This should never happen since we already validated the URL
            // but we need to handle the potential error
            crate::r#async::Jira::new("http://localhost", Credentials::Anonymous).unwrap()
        }
    }
}
