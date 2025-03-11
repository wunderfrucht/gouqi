use tracing::debug;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::ClientCore;
use crate::rep::Session;
use crate::{Credentials, Result};

/// Entrypoint into async client interface
/// <https://docs.atlassian.com/jira/REST/latest/>
#[derive(Clone, Debug)]
pub struct Jira {
    pub(crate) core: ClientCore,
    client: Client,
}

// Access methods to maintain compatibility
impl Jira {
    // This method is only used with the synchronous Jira client
    // Adding cfg attribute to prevent dead code warning when using async feature
    #[allow(dead_code)]
    pub(crate) fn host(&self) -> &url::Url {
        &self.core.host
    }
}

impl Jira {
    /// Creates a new instance of an async jira client
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Jira>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        Ok(Jira {
            core,
            client: Client::new(),
        })
    }

    /// Creates a new instance of an async jira client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jira>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        Ok(Jira { core, client })
    }

    /// Creates an async client instance directly from an existing ClientCore
    ///
    /// This is particularly useful for converting between sync and async clients
    /// while preserving all configuration and credentials.
    ///
    /// # Arguments
    ///
    /// * `core` - An existing ClientCore instance containing host and credentials
    ///
    /// # Returns
    ///
    /// A `Result` containing the new async Jira client instance if successful
    pub fn with_core(core: ClientCore) -> Result<Jira> {
        Ok(Jira {
            core,
            client: Client::new(),
        })
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

    /// Asynchronously retrieves the current user's session information from Jira
    ///
    /// This method provides information about the authenticated user's session,
    /// including user details and authentication status.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Session` information if successful, or an
    /// `Error` if the request fails
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
        let data = self.core.prepare_json_body(body)?;
        debug!("Json POST request sent");
        self.request::<D>(Method::POST, api_name, endpoint, Some(data))
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
        let data = self.core.prepare_json_body(body)?;
        debug!("Json PUT request sent");
        self.request::<D>(Method::PUT, api_name, endpoint, Some(data))
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
        let url = self.core.build_url(api_name, endpoint)?;
        debug!("url -> {:?}", url);

        let mut req = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json");

        req = self.core.apply_credentials_async(req);

        if let Some(body) = body {
            req = req.body(body);
        }
        debug!("req '{:?}'", req);

        let res = req.send().await?;
        let status = res.status();
        let body = res.text().await?;

        debug!("status {:?} body '{:?}'", status, body);

        self.core.process_response(status, &body)
    }
}

// Convert an async Jira instance to a sync one
impl From<&Jira> for crate::sync::Jira {
    fn from(async_jira: &Jira) -> Self {
        // Using the ClientCore directly is more reliable than trying to recreate it
        if let Ok(jira) = crate::sync::Jira::with_core(async_jira.core.clone()) {
            jira
        } else {
            // This fallback should never be needed since we're reusing the core
            crate::sync::Jira::new("http://localhost", Credentials::Anonymous).unwrap()
        }
    }
}
