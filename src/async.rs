use tracing::debug;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::rep::Session;
use crate::{Credentials, Error, Errors, Result};

/// Entrypoint into async client interface
/// <https://docs.atlassian.com/jira/REST/latest/>
#[derive(Clone, Debug)]
pub struct Jira {
    pub(crate) host: Url,
    pub(crate) credentials: Credentials,
    client: Client,
}

impl Jira {
    /// Creates a new instance of an async jira client
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Jira>
    where
        H: Into<String>,
    {
        match Url::parse(&host.into()) {
            Ok(host) => Ok(Jira {
                host,
                client: Client::new(),
                credentials,
            }),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Creates a new instance of an async jira client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jira>
    where
        H: Into<String>,
    {
        match Url::parse(&host.into()) {
            Ok(host) => Ok(Jira {
                host,
                client,
                credentials,
            }),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Return search interface
    #[tracing::instrument]
    pub fn search(&self) -> crate::search::AsyncSearch {
        crate::search::AsyncSearch::new(self)
    }

    /// Return issues interface
    #[tracing::instrument]
    pub fn issues(&self) -> crate::issues::AsyncIssues {
        crate::issues::AsyncIssues::new(self)
    }

    pub async fn session(&self) -> Result<Session> {
        self.get("auth", "/session").await
    }

    /// Sends a DELETE request using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    #[tracing::instrument]
    pub async fn delete<D>(&self, api_name: &str, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::DELETE, api_name, endpoint, None)
            .await
    }

    /// Sends a GET request using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    #[tracing::instrument]
    pub async fn get<D>(&self, api_name: &str, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::GET, api_name, endpoint, None)
            .await
    }

    /// Sends a POST request using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    pub async fn post<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json POST request: {}", data);
        self.request::<D>(Method::POST, api_name, endpoint, Some(data.into_bytes()))
            .await
    }

    /// Sends a PUT request using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    pub async fn put<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json request: {}", data);
        self.request::<D>(Method::PUT, api_name, endpoint, Some(data.into_bytes()))
            .await
    }

    #[tracing::instrument]
    async fn request<D>(
        &self,
        method: Method,
        api_name: &str,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let url = self
            .host
            .join(&format!("rest/{api_name}/latest{endpoint}"))?;
        debug!("url -> {:?}", url);

        let mut req = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json");

        // Apply credentials to the request
        req = match &self.credentials {
            Credentials::Anonymous => req,
            Credentials::Basic(ref user, ref pass) => {
                req.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(ref token) => req.bearer_auth(token.to_owned()),
        };

        if let Some(body) = body {
            req = req.body(body);
        }
        debug!("req '{:?}'", req);

        let res = req.send().await?;
        let status = res.status();
        let body = res.text().await?;

        debug!("status {:?} body '{:?}'", status, body);

        match status {
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::METHOD_NOT_ALLOWED => Err(Error::MethodNotAllowed),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            client_err if client_err.is_client_error() => Err(Error::Fault {
                code: status,
                errors: serde_json::from_str::<Errors>(&body)?,
            }),
            _ => {
                let data = if body.is_empty() { "null" } else { &body };
                Ok(serde_json::from_str::<D>(data)?)
            }
        }
    }
}

// Convert an async Jira instance to a sync one
impl From<&Jira> for crate::sync::Jira {
    fn from(async_jira: &Jira) -> Self {
        // This is a best-effort conversion - if it fails, we'll just create a new client
        if let Ok(jira) =
            crate::sync::Jira::new(async_jira.host.as_str(), async_jira.credentials.clone())
        {
            jira
        } else {
            // This should never happen since we already validated the URL
            // but we need to handle the potential error
            crate::sync::Jira::new("http://localhost", Credentials::Anonymous).unwrap()
        }
    }
}
