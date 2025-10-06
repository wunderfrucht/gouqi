use std::io::Read;
use tracing::debug;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Method, blocking::Client};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::attachments::Attachments;
use crate::boards::Boards;
use crate::components::Components;
use crate::core::{ClientCore, RequestContext};
use crate::groups::Groups;
use crate::issues::Issues;
use crate::projects::Projects;
use crate::rep::Session;
use crate::resolution::Resolution;
use crate::search::Search;
use crate::sprints::Sprints;
use crate::transitions::Transitions;
use crate::users::Users;
use crate::versions::Versions;
use crate::{Credentials, Error, Result};

/// Entrypoint into client interface
/// <https://docs.atlassian.com/jira/REST/latest/>
#[derive(Clone, Debug)]
pub struct Jira {
    pub(crate) core: ClientCore,
    client: Client,
}

// Access methods to maintain compatibility
impl Jira {
    pub(crate) fn host(&self) -> &url::Url {
        &self.core.host
    }

    /// Get the configured search API version for testing purposes
    #[cfg(debug_assertions)]
    pub fn get_search_api_version(&self) -> crate::core::SearchApiVersion {
        self.core.get_search_api_version()
    }
}

impl Jira {
    /// Creates a new instance of a jira client
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

    /// Creates a new instance of a jira client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jira>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        Ok(Jira { core, client })
    }

    /// Creates a client instance directly from an existing ClientCore
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
    /// A `Result` containing the new Jira client instance if successful
    pub fn with_core(core: ClientCore) -> Result<Jira> {
        Ok(Jira {
            core,
            client: Client::new(),
        })
    }

    /// Creates a new Jira client with specific search API version
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
    /// A `Result` containing the new Jira client instance if successful
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

    /// Return transitions interface
    pub fn transitions<K>(&self, key: K) -> Transitions
    where
        K: Into<String>,
    {
        Transitions::new(self, key)
    }

    /// Return search interface
    #[tracing::instrument]
    pub fn search(&self) -> Search {
        Search::new(self)
    }

    /// Returns the issues interface for working with Jira issues
    ///
    /// This interface provides methods to create, read, update, and delete issues,
    /// as well as operations for working with issue fields, comments, and other
    /// issue-related data.
    ///
    /// # Returns
    ///
    /// An `Issues` instance configured with this client
    #[tracing::instrument]
    pub fn issues(&self) -> Issues {
        Issues::new(self)
    }

    /// Returns the issue links interface for managing links between issues
    ///
    /// Issue links represent relationships between issues (e.g., "blocks", "relates to").
    /// This interface provides methods to create, retrieve, and delete issue links.
    ///
    /// # Returns
    ///
    /// An `IssueLinks` instance configured with this client
    #[tracing::instrument]
    pub fn issue_links(&self) -> crate::issue_links::IssueLinks {
        crate::issue_links::IssueLinks::new(self)
    }

    /// Returns the projects interface for working with Jira projects
    ///
    /// Projects in Jira contain issues and define the scope of work. This interface
    /// provides methods to create, retrieve, update, and delete projects, as well as
    /// manage project components, versions, and roles.
    ///
    /// # Returns
    ///
    /// A `Projects` instance configured with this client
    #[tracing::instrument]
    pub fn projects(&self) -> Projects {
        Projects::new(self)
    }

    /// Returns the attachments interface for working with Jira issue attachments
    ///
    /// This interface allows managing file attachments on Jira issues,
    /// providing methods to retrieve metadata about attachments and
    /// manage attachment content.
    ///
    /// # Returns
    ///
    /// An `Attachments` instance configured with this client
    pub fn attachments(&self) -> Attachments {
        Attachments::new(self)
    }

    /// Returns the components interface for working with Jira project components
    ///
    /// Components are used in Jira to group issues within a project. This interface
    /// provides methods to create, retrieve, update, and delete project components.
    ///
    /// # Returns
    ///
    /// A `Components` instance configured with this client
    pub fn components(&self) -> Components {
        Components::new(self)
    }

    /// Returns the boards interface for working with Jira Agile boards
    ///
    /// Boards in Jira Agile provide a visual way to manage work. This interface
    /// allows interaction with boards, including retrieving board information,
    /// sprints, and issues on boards.
    ///
    /// # Returns
    ///
    /// A `Boards` instance configured with this client
    #[tracing::instrument]
    pub fn boards(&self) -> Boards {
        Boards::new(self)
    }

    /// Returns the sprints interface for working with Jira Agile sprints
    ///
    /// Sprints are time-boxed iterations in Jira Agile. This interface provides
    /// methods to access sprint data, create or update sprints, and manage
    /// the issues within sprints.
    ///
    /// # Returns
    ///
    /// A `Sprints` instance configured with this client
    #[tracing::instrument]
    pub fn sprints(&self) -> Sprints {
        Sprints::new(self)
    }

    /// Returns the versions interface for working with Jira project versions
    ///
    /// Versions represent releases or milestones in Jira projects. This interface
    /// allows creating, retrieving, updating, and deleting project versions,
    /// as well as managing issues associated with versions.
    ///
    /// # Returns
    ///
    /// A `Versions` instance configured with this client
    #[tracing::instrument]
    pub fn versions(&self) -> Versions {
        Versions::new(self)
    }

    /// Returns the resolution interface for working with Jira issue resolutions
    ///
    /// Resolutions represent the outcome of an issue when it is closed.
    /// This interface allows retrieving resolution information.
    ///
    /// # Returns
    ///
    /// A `Resolution` instance configured with this client
    #[tracing::instrument]
    pub fn resolution(&self) -> Resolution {
        Resolution::new(self)
    }

    /// Returns the users interface for working with Jira users
    ///
    /// This interface provides methods to search for users, get user details,
    /// and find users assignable to projects and issues.
    ///
    /// # Returns
    ///
    /// A `Users` instance configured with this client
    #[tracing::instrument]
    pub fn users(&self) -> Users {
        Users::new(self)
    }

    /// Returns the groups interface for working with Jira groups
    ///
    /// This interface provides methods to list groups, get group members,
    /// create and delete groups, and manage group membership.
    ///
    /// # Returns
    ///
    /// A `Groups` instance configured with this client
    #[tracing::instrument]
    pub fn groups(&self) -> Groups {
        Groups::new(self)
    }

    /// Retrieves the current user's session information from Jira
    ///
    /// This method provides information about the authenticated user's session,
    /// including user details and authentication status.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Session` information if successful, or an
    /// `Error` if the request fails
    pub fn session(&self) -> Result<Session> {
        self.get("auth", "/session")
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

    /// Sends a DELETE request using the Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::EmptyResponse;
    /// # use gouqi::Credentials;
    /// # use gouqi::Jira;
    /// # let jira = Jira::new("http://localhost".to_string(), Credentials::Anonymous).unwrap();
    /// let response = jira.delete::<EmptyResponse>("api", "/endpoint");
    /// ```
    #[tracing::instrument]
    pub fn delete<D>(&self, api_name: &str, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::DELETE, api_name, endpoint, None)
    }

    /// Sends a GET request using the Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    ///
    /// # Examples
    ///
    /// ```rust    
    /// # use gouqi::EmptyResponse;
    /// # use gouqi::Credentials;
    /// # use gouqi::Jira;
    /// # let jira = Jira::new("http://localhost".to_string(), Credentials::Anonymous).unwrap();
    /// let response = jira.get::<EmptyResponse>("api", "/endpoint");
    /// ```
    #[tracing::instrument]
    pub fn get<D>(&self, api_name: &str, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::GET, api_name, endpoint, None)
    }

    /// Sends a GET request with specific API version using the Jira client.
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
    pub fn get_versioned<D>(
        &self,
        api_name: &str,
        version: Option<&str>,
        endpoint: &str,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request_versioned::<D>(Method::GET, api_name, version, endpoint, None)
    }

    /// Sends a POST request with a specific API version using the Jira client.
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
    pub fn post_versioned<D, S>(
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
    pub fn get_bytes(&self, api_name: &str, endpoint: &str) -> Result<Vec<u8>> {
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

        req = self.core.apply_credentials_sync(req);

        debug!(
            correlation_id = %ctx.correlation_id,
            "Sending bytes request"
        );

        let result = (|| {
            let mut res = req.send()?;
            let status = res.status();

            if !status.is_success() {
                let mut response_body = String::new();
                res.read_to_string(&mut response_body)?;

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

            let mut bytes = Vec::new();
            res.read_to_end(&mut bytes)?;

            debug!(
                correlation_id = %ctx.correlation_id,
                status = %status,
                bytes_size = bytes.len(),
                "Received bytes response"
            );

            Ok(bytes)
        })();

        let success = result.is_ok();
        ctx.finish(success);

        result
    }

    /// Sends a POST request using the Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::EmptyResponse;
    /// # use serde::Serialize;
    /// # use gouqi::Credentials;
    /// # use gouqi::Jira;
    /// #[derive(Serialize, Debug, Default)]
    /// struct EmptyBody;
    ///
    /// # let jira = Jira::new("http://localhost".to_string(), Credentials::Anonymous).unwrap();
    /// let body = EmptyBody::default();
    /// let response = jira.post::<EmptyResponse, EmptyBody>("api", "/endpoint", body);
    /// ```
    pub fn post<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = self.core.prepare_json_body(body)?;
        debug!("Json POST request sent");
        self.request::<D>(Method::POST, api_name, endpoint, Some(data))
    }

    /// Sends a PUT request using the Jira client.
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns  
    ///
    /// `Result<D>` - Response deserialized into type `D`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::EmptyResponse;
    /// # use serde::Serialize;
    /// # use gouqi::Credentials;
    /// # use gouqi::Jira;
    /// #[derive(Serialize, Debug, Default)]
    /// struct EmptyBody;
    ///
    /// # let jira = Jira::new("http://localhost".to_string(), Credentials::Anonymous).unwrap();
    /// let body = EmptyBody::default();
    /// let response = jira.put::<EmptyResponse, EmptyBody>("api", "/endpoint", body);
    /// ```
    pub fn put<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = self.core.prepare_json_body(body)?;
        debug!("Json PUT request sent");
        self.request::<D>(Method::PUT, api_name, endpoint, Some(data))
    }

    /// Sends a POST request with multipart/form-data
    ///
    /// # Arguments
    ///
    /// * `api_name` - Name of the API: like "agile" or "api"
    /// * `endpoint` - API endpoint path
    /// * `form` - Multipart form data
    ///
    /// # Returns
    ///
    /// `Result<D>` - Response deserialized into type `D`
    pub fn post_multipart<D>(
        &self,
        api_name: &str,
        endpoint: &str,
        form: reqwest::blocking::multipart::Form,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let ctx = RequestContext::new("POST", endpoint);
        let _span = ctx.create_span().entered();

        let url = self.core.build_url(api_name, endpoint)?;
        debug!(
            correlation_id = %ctx.correlation_id,
            url = %url,
            "Building multipart request URL"
        );

        // Generate OAuth header if using OAuth 1.0a
        #[cfg(feature = "oauth")]
        let oauth_header = self.core.get_oauth_header("POST", url.as_str())?;

        let mut req = self
            .client
            .request(Method::POST, url)
            .header("X-Atlassian-Token", "no-check")
            .header("X-Correlation-ID", &ctx.correlation_id)
            .multipart(form);

        // Apply OAuth header if present
        #[cfg(feature = "oauth")]
        if let Some(header) = oauth_header {
            req = req.header(reqwest::header::AUTHORIZATION, header);
        }

        req = self.core.apply_credentials_sync(req);

        debug!(
            correlation_id = %ctx.correlation_id,
            "Sending multipart request"
        );

        (|| {
            let mut res = req.send()?;
            let status = res.status();

            let mut response_body = String::new();
            res.read_to_string(&mut response_body)?;

            debug!(
                correlation_id = %ctx.correlation_id,
                status = %status,
                response_size = response_body.len(),
                "Received response"
            );

            let response = self.core.process_response(status, &response_body)?;

            ctx.finish(false);

            Ok(response)
        })()
    }

    #[tracing::instrument(skip(self, body))]
    fn request<D>(
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
        let _span = ctx.create_span().entered();
        #[allow(unused_variables)]
        let method_str = method.to_string();

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

        req = self.core.apply_credentials_sync(req);

        if let Some(body) = body {
            req = req.body(body);
        }

        debug!(
            correlation_id = %ctx.correlation_id,
            "Sending request"
        );

        let result = (|| {
            let mut res = req.send()?;
            let status = res.status();

            let mut response_body = String::new();
            res.read_to_string(&mut response_body)?;

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
        })();

        let success = result.is_ok();
        ctx.finish(success);

        result
    }

    #[tracing::instrument(skip(self, body))]
    fn request_versioned<D>(
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
        let _span = ctx.create_span().entered();
        #[allow(unused_variables)]
        let method_str = method.to_string();

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

        req = self.core.apply_credentials_sync(req);

        if let Some(body) = body {
            req = req.body(body);
        }

        debug!(
            correlation_id = %ctx.correlation_id,
            "Sending versioned request"
        );

        let result = (|| {
            let mut res = req.send()?;
            let status = res.status();

            let mut response_body = String::new();
            res.read_to_string(&mut response_body)?;

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
        })();

        let success = result.is_ok();
        ctx.finish(success);

        result
    }
}
