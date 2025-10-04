//! Asynchronous API for interacting with Jira.
//!
//! This module provides an asynchronous client for the Jira REST API. It is only available
//! when the `async` feature is enabled.
//!
//! # Usage
//!
//! ```no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use gouqi::{Credentials, r#async::Jira, SearchOptions};
//!
//! let credentials = Credentials::Basic("username".to_string(), "password".to_string());
//! let jira = Jira::new("https://jira.example.com", credentials)?;
//!
//! // Get information about the current session
//! let session = jira.session().await?;
//! println!("Logged in as: {}", session.name);
//!
//! // Search for issues
//! let results = jira.search().list("project = DEMO", &Default::default()).await?;
//! println!("Found {} issues", results.total);
//!
//! // Get board information
//! let board = jira.boards().get(42u64).await?;
//! println!("Board name: {}", board.name);
//!
//! // List all boards
//! let options = SearchOptions::default();
//! let boards = jira.boards().list(&options).await?;
//! println!("Found {} boards", boards.values.len());
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Authentication Methods
//!
//! The async client supports the same authentication methods as the sync client:
//!
//! - Anonymous: `Credentials::Anonymous`
//! - Basic authentication: `Credentials::Basic(username, password)`
//! - Bearer token: `Credentials::Bearer(token)`
//! - Cookie-based: `Credentials::Cookie(jsessionid)`

use tracing::{Instrument, debug};

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::core::{ClientCore, RequestContext};
use crate::rep::Session;
use crate::{Credentials, Error, Result};

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

    /// Creates a new async Jira client with specific search API version
    ///
    /// This allows you to explicitly control which search API version to use:
    /// - `SearchApiVersion::Auto` (default): Automatically detect best version
    /// - `SearchApiVersion::V2`: Force use of legacy /rest/api/2/search
    /// - `SearchApiVersion::V3`: Force use of enhanced /rest/api/3/search/jql
    ///
    /// # Arguments
    ///
    /// * `host` - Jira server URL
    /// * `credentials` - Authentication credentials
    /// * `search_api_version` - Which search API version to use
    ///
    /// # Returns
    ///
    /// A `Result` containing the new async Jira client instance if successful
    pub fn with_search_api_version<H>(
        host: H,
        credentials: Credentials,
        search_api_version: crate::core::SearchApiVersion,
    ) -> Result<Jira>
    where
        H: Into<String>,
    {
        let core = ClientCore::with_search_api_version(host, credentials, search_api_version)?;
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

    /// Get the configured search API version for testing purposes
    #[cfg(debug_assertions)]
    pub fn get_search_api_version(&self) -> crate::core::SearchApiVersion {
        self.core.get_search_api_version()
    }

    /// Return issues interface
    #[tracing::instrument]
    pub fn issues(&self) -> crate::issues::AsyncIssues {
        crate::issues::AsyncIssues::new(self)
    }

    /// Returns the issue links interface for managing links between issues asynchronously
    ///
    /// # Returns
    ///
    /// An `AsyncIssueLinks` instance configured with this client
    #[tracing::instrument]
    pub fn issue_links(&self) -> crate::issue_links::AsyncIssueLinks {
        crate::issue_links::AsyncIssueLinks::new(self)
    }

    /// Returns the projects interface for working with Jira projects asynchronously
    ///
    /// Projects in Jira contain issues and define the scope of work. This interface
    /// provides methods to create, retrieve, update, and delete projects, as well as
    /// manage project components, versions, and roles.
    ///
    /// # Returns
    ///
    /// An `AsyncProjects` instance configured with this client
    #[tracing::instrument]
    pub fn projects(&self) -> crate::projects::AsyncProjects {
        crate::projects::AsyncProjects::new(self)
    }

    /// Returns the boards interface for working with Jira Agile boards asynchronously
    ///
    /// Boards in Jira Agile provide a visual way to manage work. This interface
    /// allows interaction with boards, including retrieving board information
    /// and listing all boards with pagination support.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use gouqi::{Credentials, r#async::Jira, SearchOptions};
    /// # use futures::stream::TryStreamExt;
    /// # let jira = Jira::new("https://jira.example.com", Credentials::Anonymous)?;
    /// // Get a specific board by ID
    /// let board = jira.boards().get(42u64).await?;
    /// println!("Board: {}", board.name);
    ///
    /// // List all boards with pagination
    /// let options = SearchOptions::default();
    /// let board_results = jira.boards().list(&options).await?;
    /// for board in board_results.values {
    ///     println!("Found board: {} ({})", board.name, board.id);
    /// }
    ///
    /// // Use streaming API for efficient pagination
    /// let boards = jira.boards();
    /// let mut stream = boards.stream(&options).await?;
    /// while let Some(board) = stream.try_next().await? {
    ///     println!("Streamed board: {}", board.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// An `AsyncBoards` instance configured with this client
    #[tracing::instrument]
    pub fn boards(&self) -> crate::boards::AsyncBoards<'_> {
        crate::boards::AsyncBoards::new(self)
    }

    /// Returns the attachments interface for working with Jira attachments
    ///
    /// # Returns
    ///
    /// An `AsyncAttachments` instance configured with this client
    #[tracing::instrument]
    pub fn attachments(&self) -> crate::attachments::AsyncAttachments {
        crate::attachments::AsyncAttachments::new(self)
    }

    /// Returns the components interface for working with Jira project components
    ///
    /// # Returns
    ///
    /// An `AsyncComponents` instance configured with this client
    #[tracing::instrument]
    pub fn components(&self) -> crate::components::AsyncComponents {
        crate::components::AsyncComponents::new(self)
    }

    /// Returns the versions interface for working with Jira project versions
    ///
    /// # Returns
    ///
    /// An `AsyncVersions` instance configured with this client
    #[tracing::instrument]
    pub fn versions(&self) -> crate::versions::AsyncVersions {
        crate::versions::AsyncVersions::new(self)
    }

    /// Returns the sprints interface for working with Jira sprints
    ///
    /// # Returns
    ///
    /// An `AsyncSprints` instance configured with this client
    #[tracing::instrument]
    pub fn sprints(&self) -> crate::sprints::AsyncSprints {
        crate::sprints::AsyncSprints::new(self)
    }

    /// Returns the transitions interface for a specific issue
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The key of the issue (e.g., "PROJ-123")
    ///
    /// # Returns
    ///
    /// An `AsyncTransitions` instance configured for the specified issue
    #[tracing::instrument]
    pub fn transitions<K>(&self, issue_key: K) -> crate::transitions::AsyncTransitions
    where
        K: Into<String> + std::fmt::Debug,
    {
        crate::transitions::AsyncTransitions::new(self, issue_key)
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

    /// Clear all cached responses
    ///
    /// This method clears all cached API responses. Useful for forcing fresh data
    /// or freeing up memory used by the cache.
    #[cfg(feature = "cache")]
    pub fn clear_cache(&self) {
        self.core.clear_cache();
    }

    /// Get cache statistics
    ///
    /// Returns statistics about the current cache state including number of entries,
    /// hit rate, and memory usage.
    ///
    /// # Returns
    ///
    /// A `CacheStats` struct containing cache performance metrics
    #[cfg(feature = "cache")]
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.core.cache_stats()
    }

    /// Get comprehensive observability report
    ///
    /// Returns a detailed report including health status, metrics, cache performance,
    /// and system information useful for monitoring and debugging.
    ///
    /// # Returns
    ///
    /// An `ObservabilityReport` containing all observability data
    #[cfg(any(feature = "metrics", feature = "cache"))]
    pub fn observability_report(&self) -> crate::observability::ObservabilityReport {
        let obs = self.create_observability_system();
        obs.get_observability_report()
    }

    /// Get health status
    ///
    /// Returns the current health status of the client including metrics
    /// and cache performance indicators.
    ///
    /// # Returns
    ///
    /// A `HealthStatus` indicating the current system state
    #[cfg(any(feature = "metrics", feature = "cache"))]
    pub fn health_status(&self) -> crate::observability::HealthStatus {
        let obs = self.create_observability_system();
        obs.health_status()
    }

    #[cfg(any(feature = "metrics", feature = "cache"))]
    fn create_observability_system(&self) -> crate::observability::ObservabilitySystem {
        #[cfg(feature = "cache")]
        {
            crate::observability::ObservabilitySystem::with_cache(self.core.cache.clone())
        }
        #[cfg(not(feature = "cache"))]
        {
            crate::observability::ObservabilitySystem::new()
        }
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

    /// Sends a GET request with specific API version using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `version` - API version: like "2", "3", "latest", or None for default
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    #[tracing::instrument]
    pub async fn get_versioned<D>(
        &self,
        api_name: &str,
        version: Option<&str>,
        endpoint: &str,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request_versioned::<D>(Method::GET, api_name, version, endpoint, None)
            .await
    }

    /// Sends a POST request with a specific API version using the async Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `version` - API version to use (e.g., Some("3"), Some("latest"), None for default)
    /// * `endpoint` - API endpoint path
    /// * `body` - Request body to serialize and send
    ///
    /// # Returns
    ///
    /// `Result<D>` - Response deserialized into type `D`
    pub async fn post_versioned<D, S>(
        &self,
        api_name: &str,
        version: Option<&str>,
        endpoint: &str,
        body: S,
    ) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = self.core.prepare_json_body(body)?;
        debug!("Json POST request sent with API version {:?}", version);
        self.request_versioned::<D>(Method::POST, api_name, version, endpoint, Some(data))
            .await
    }

    /// Sends a GET request and returns raw bytes (for downloading files)
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns
    ///
    /// `Result<Vec<u8>>` - Raw response bytes
    pub async fn get_bytes(&self, api_name: &str, endpoint: &str) -> Result<Vec<u8>> {
        let ctx = RequestContext::new("GET", endpoint);
        let _span = ctx.create_span().entered();

        let url = self.core.build_url(api_name, endpoint)?;
        debug!(
            correlation_id = %ctx.correlation_id,
            url = %url,
            "Building request URL for bytes download"
        );

        let mut req = self
            .client
            .request(Method::GET, url)
            .header("X-Correlation-ID", &ctx.correlation_id);

        req = self.core.apply_credentials_async(req);

        debug!(
            correlation_id = %ctx.correlation_id,
            "Sending bytes request"
        );

        let result = async {
            let res = req.send().await?;
            let status = res.status();

            if !status.is_success() {
                let response_body = res.text().await?;

                debug!(
                    correlation_id = %ctx.correlation_id,
                    status = %status,
                    response_size = response_body.len(),
                    "Received error response"
                );

                return Err(match status {
                    reqwest::StatusCode::UNAUTHORIZED => Error::Unauthorized,
                    reqwest::StatusCode::METHOD_NOT_ALLOWED => Error::MethodNotAllowed,
                    reqwest::StatusCode::NOT_FOUND => Error::NotFound,
                    client_err if client_err.is_client_error() => Error::Fault {
                        code: status,
                        errors: serde_json::from_str(&response_body)?,
                    },
                    _ => Error::Fault {
                        code: status,
                        errors: serde_json::from_str(&response_body)?,
                    },
                });
            }

            let bytes = res.bytes().await?.to_vec();

            debug!(
                correlation_id = %ctx.correlation_id,
                status = %status,
                bytes_size = bytes.len(),
                "Received bytes response"
            );

            Ok(bytes)
        }
        .await;

        let success = result.is_ok();
        ctx.finish(success);

        result
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

    #[tracing::instrument(skip(self, body))]
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
        let ctx = RequestContext::new(method.as_ref(), endpoint);
        let span = ctx.create_span();
        let method_str = method.to_string();

        async move {
            // Check cache first for GET requests
            #[cfg(feature = "cache")]
            if method == Method::GET {
                if let Some(cached_response) = self.core.check_cache::<D>(&method_str, endpoint) {
                    debug!(
                        correlation_id = %ctx.correlation_id,
                        endpoint = endpoint,
                        "Returning cached response"
                    );
                    ctx.finish(true);
                    return Ok(cached_response);
                }
            }

            let url = self.core.build_url(api_name, endpoint)?;
            debug!(
                correlation_id = %ctx.correlation_id,
                url = %url,
                "Building request URL"
            );

            // Generate OAuth header if using OAuth 1.0a
            #[cfg(feature = "oauth")]
            let oauth_header = self.core.get_oauth_header(method.as_str(), url.as_str())?;

            let mut req = self
                .client
                .request(method.clone(), url)
                .header(CONTENT_TYPE, "application/json")
                .header("X-Correlation-ID", &ctx.correlation_id);

            // Apply OAuth header if present
            #[cfg(feature = "oauth")]
            if let Some(header) = oauth_header {
                req = req.header(reqwest::header::AUTHORIZATION, header);
            }

            req = self.core.apply_credentials_async(req);

            if let Some(body) = body {
                req = req.body(body);
            }

            debug!(
                correlation_id = %ctx.correlation_id,
                "Sending request"
            );

            let result = async {
                let res = req.send().await?;
                let status = res.status();
                let response_body = res.text().await?;

                debug!(
                    correlation_id = %ctx.correlation_id,
                    status = %status,
                    response_size = response_body.len(),
                    "Received response"
                );

                let response = self.core.process_response(status, &response_body)?;

                // Cache successful GET responses by storing the raw JSON
                #[cfg(feature = "cache")]
                if status.is_success() && method == Method::GET {
                    self.core
                        .store_raw_response(&method_str, endpoint, &response_body);
                }

                Ok(response)
            }
            .await;

            let success = result.is_ok();
            ctx.finish(success);

            result
        }
        .instrument(span)
        .await
    }

    #[tracing::instrument(skip(self, body))]
    async fn request_versioned<D>(
        &self,
        method: Method,
        api_name: &str,
        version: Option<&str>,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let ctx = RequestContext::new(method.as_ref(), endpoint);
        let span = ctx.create_span();
        #[allow(unused_variables)]
        let method_str = method.to_string();

        let result = async {
            // Check cache first for GET requests
            #[cfg(feature = "cache")]
            if method == Method::GET {
                if let Some(cached_response) = self.core.check_cache::<D>(&method_str, endpoint) {
                    debug!(
                        correlation_id = %ctx.correlation_id,
                        endpoint = endpoint,
                        "Returning cached response"
                    );
                    ctx.finish(true);
                    return Ok(cached_response);
                }
            }

            let url = self.core.build_versioned_url(api_name, version, endpoint)?;
            debug!(
                correlation_id = %ctx.correlation_id,
                url = %url,
                "Building versioned request URL"
            );

            let mut req = self
                .client
                .request(method.clone(), url)
                .header(CONTENT_TYPE, "application/json")
                .header("X-Correlation-ID", &ctx.correlation_id);

            req = self.core.apply_credentials_async(req);

            if let Some(body) = body {
                req = req.body(body);
            }

            debug!(
                correlation_id = %ctx.correlation_id,
                "Sending versioned request"
            );

            let res = req.send().await?;
            let status = res.status();
            let response_body = res.text().await?;

            debug!(
                correlation_id = %ctx.correlation_id,
                status = %status,
                response_size = response_body.len(),
                "Received versioned response"
            );

            let response = self.core.process_response(status, &response_body)?;

            // Cache successful GET responses by storing the raw JSON
            #[cfg(feature = "cache")]
            if status.is_success() && method == Method::GET {
                self.core
                    .store_raw_response(&method_str, endpoint, &response_body);
            }

            Ok(response)
        }
        .instrument(span)
        .await;

        let success = result.is_ok();
        ctx.finish(success);

        result
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
