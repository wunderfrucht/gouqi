//! Core shared functionality between sync and async implementations

use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::debug;
use url::Url;

use crate::Error;

/// Type alias for Result with the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// An empty response structure, used for endpoints that return no data
#[derive(Deserialize, Debug)]
pub struct EmptyResponse;

/// Types of authentication credentials
///
/// # Notes
///
/// - Personal Access Token are used with [`Credentials::Basic`] scheme as a password replacement and *not* as a [`Credentials::Bearer`]
///   like the [API documentation suggests](https://developer.atlassian.com/server/jira/platform/rest-apis/#authentication-and-authorization).
/// - Cookie-based authentication (`Credentials::Cookie`) uses the JSESSIONID cookie as described in 
///   [Cookie-based Authentication](https://developer.atlassian.com/server/jira/platform/cookie-based-authentication/).
#[derive(Clone, Debug)]
pub enum Credentials {
    /// Use no authentication
    Anonymous,
    /// Username and password credentials (Personal Access Token count as a password)
    Basic(String, String),
    /// Authentication via bearer token
    Bearer(String),
    /// Cookie-based authentication using JSESSIONID
    Cookie(String),
    // TODO: Add OAuth
}

/// Common data required for both sync and async clients
#[derive(Clone, Debug)]
pub struct ClientCore {
    pub host: Url,
    pub credentials: Credentials,
}

impl ClientCore {
    /// Creates a new client core with the given host and credentials
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Self>
    where
        H: Into<String>,
    {
        match Url::parse(&host.into()) {
            Ok(host) => Ok(ClientCore { host, credentials }),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Builds the API URL for a request
    pub fn build_url(&self, api_name: &str, endpoint: &str) -> Result<Url> {
        self.host
            .join(&format!("rest/{api_name}/latest{endpoint}"))
            .map_err(Error::from)
    }

    /// Prepares JSON body data from a serializable object
    pub fn prepare_json_body<S>(&self, body: S) -> Result<Vec<u8>>
    where
        S: Serialize,
    {
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json request: {}", data);
        Ok(data.into_bytes())
    }

    /// Process the response body and convert it into the specified type
    pub fn process_response<D>(&self, status: reqwest::StatusCode, body: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        match status {
            reqwest::StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            reqwest::StatusCode::METHOD_NOT_ALLOWED => Err(Error::MethodNotAllowed),
            reqwest::StatusCode::NOT_FOUND => Err(Error::NotFound),
            client_err if client_err.is_client_error() => Err(Error::Fault {
                code: status,
                errors: serde_json::from_str(body)?,
            }),
            _ => {
                let data = if body.is_empty() { "null" } else { body };
                Ok(serde_json::from_str::<D>(data)?)
            }
        }
    }

    /// Apply credentials to a sync request builder
    pub fn apply_credentials_sync(
        &self,
        builder: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match &self.credentials {
            Credentials::Anonymous => builder,
            Credentials::Basic(ref user, ref pass) => {
                builder.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(ref token) => builder.bearer_auth(token.to_owned()),
            Credentials::Cookie(ref jsessionid) => {
                builder.header(reqwest::header::COOKIE, format!("JSESSIONID={}", jsessionid))
            }
        }
    }

    /// Apply credentials to an async request builder
    pub fn apply_credentials_async(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        match &self.credentials {
            Credentials::Anonymous => builder,
            Credentials::Basic(ref user, ref pass) => {
                builder.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(ref token) => builder.bearer_auth(token.to_owned()),
            Credentials::Cookie(ref jsessionid) => {
                builder.header(reqwest::header::COOKIE, format!("JSESSIONID={}", jsessionid))
            }
        }
    }
}

/// Shared trait for pagination functionality between sync and async implementations
///
/// This trait provides common pagination logic used by both the synchronous iterator
/// and asynchronous streams when handling paginated API responses.
pub trait PaginationInfo {
    /// Determines if more pages of results exist beyond the current page
    ///
    /// # Arguments
    ///
    /// * `count` - Number of items in the current page
    /// * `start_at` - The starting index of the current page
    /// * `total` - The total number of results available
    ///
    /// # Returns
    ///
    /// `true` if more pages are available, `false` otherwise
    fn more_pages(count: u64, start_at: u64, total: u64) -> bool {
        (start_at + count) < total
    }
}
