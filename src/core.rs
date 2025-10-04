//! Core shared functionality between sync and async implementations

#[cfg(feature = "cache")]
use std::sync::Arc;
#[cfg(feature = "metrics")]
use std::time::Instant;
#[cfg(feature = "uuid")]
use uuid::Uuid;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, info_span};
use url::Url;

use crate::Error;
#[cfg(feature = "cache")]
use crate::cache::{Cache, MemoryCache, RuntimeCacheConfig, generate_cache_key};
#[cfg(feature = "metrics")]
use crate::metrics::{METRICS, MetricsCollector};

/// Type alias for Result with the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// An empty response structure, used for endpoints that return no data
#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyResponse;

/// Jira deployment types for API version detection
#[derive(Clone, Debug, PartialEq)]
pub enum JiraDeploymentType {
    /// Jira Cloud (*.atlassian.net) - typically supports latest API versions
    Cloud,
    /// Jira Data Center - self-managed enterprise, version support varies
    DataCenter,
    /// Jira Server - end-of-life but may still exist, limited API support
    Server,
    /// Unknown deployment type - requires capability testing
    Unknown,
}

/// Search API versions supported by different Jira deployments
#[derive(Clone, Debug, PartialEq, Default)]
pub enum SearchApiVersion {
    /// Automatically detect the best available version
    #[default]
    Auto,
    /// Use /rest/api/2/search (legacy, being deprecated)
    V2,
    /// Use /rest/api/3/search/jql (enhanced search)
    V3,
}

/// Types of authentication credentials
///
/// # Notes
///
/// - Personal Access Token are used with [`Credentials::Basic`] scheme as a password replacement and *not* as a [`Credentials::Bearer`]
///   like the [API documentation suggests](https://developer.atlassian.com/server/jira/platform/rest-apis/#authentication-and-authorization).
/// - Cookie-based authentication (`Credentials::Cookie`) uses the JSESSIONID cookie as described in
///   [Cookie-based Authentication](https://developer.atlassian.com/server/jira/platform/cookie-based-authentication/).
/// - OAuth 2.0 access tokens (Jira Cloud) should use [`Credentials::Bearer`] directly
/// - OAuth 1.0a (Jira Server/Data Center) uses RSA-SHA1 request signing and requires the `oauth` feature
#[derive(Clone, Debug)]
pub enum Credentials {
    /// Use no authentication
    Anonymous,
    /// Username and password credentials (Personal Access Token count as a password)
    Basic(String, String),
    /// Authentication via bearer token (includes OAuth 2.0 access tokens for Jira Cloud)
    Bearer(String),
    /// Cookie-based authentication using JSESSIONID
    Cookie(String),
    /// OAuth 1.0a authentication for Jira Server/Data Center
    ///
    /// Uses RSA-SHA1 request signing. Requires the `oauth` feature to be enabled.
    ///
    /// # Fields
    ///
    /// - `consumer_key`: The OAuth consumer key
    /// - `private_key_pem`: RSA private key in PEM format
    /// - `access_token`: The OAuth access token
    /// - `access_token_secret`: The OAuth access token secret
    #[cfg(feature = "oauth")]
    OAuth1a {
        consumer_key: String,
        private_key_pem: String,
        access_token: String,
        access_token_secret: String,
    },
}

/// Common data required for both sync and async clients
#[derive(Clone)]
pub struct ClientCore {
    pub host: Url,
    pub credentials: Credentials,
    pub search_api_version: SearchApiVersion,
    #[cfg(feature = "cache")]
    pub cache: Arc<dyn Cache>,
    #[cfg(feature = "cache")]
    pub cache_config: RuntimeCacheConfig,
}

impl std::fmt::Debug for ClientCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientCore")
            .field("host", &self.host)
            .field("credentials", &self.credentials)
            .field("search_api_version", &self.search_api_version)
            .field("cache_enabled", &cfg!(feature = "cache"))
            .finish()
    }
}

/// Request context for tracing and metrics
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique correlation ID for this request
    pub correlation_id: String,
    /// Request method (GET, POST, etc.)
    pub method: String,
    /// API endpoint being called
    pub endpoint: String,
    /// Start time for duration tracking
    #[cfg(feature = "metrics")]
    pub start_time: Instant,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(method: &str, endpoint: &str) -> Self {
        #[cfg(feature = "uuid")]
        let correlation_id = Uuid::new_v4().to_string();
        #[cfg(not(feature = "uuid"))]
        let correlation_id = format!(
            "req_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );

        Self {
            correlation_id,
            method: method.to_string(),
            endpoint: endpoint.to_string(),
            #[cfg(feature = "metrics")]
            start_time: Instant::now(),
        }
    }

    /// Record request completion and metrics
    pub fn finish(&self, success: bool) {
        #[cfg(feature = "metrics")]
        {
            let duration = self.start_time.elapsed();
            METRICS.record_request(&self.method, &self.endpoint, duration, success);

            if !success {
                METRICS.record_error("request_failed");
            }
        }

        debug!(
            correlation_id = %self.correlation_id,
            method = %self.method,
            endpoint = %self.endpoint,
            success = success,
            "Request completed"
        );
    }

    /// Create a tracing span for this request
    pub fn create_span(&self) -> tracing::Span {
        info_span!(
            "jira_request",
            correlation_id = %self.correlation_id,
            method = %self.method,
            endpoint = %self.endpoint
        )
    }
}

impl ClientCore {
    /// Creates a new client core with the given host and credentials
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Self>
    where
        H: Into<String>,
    {
        Self::with_search_api_version(host, credentials, SearchApiVersion::default())
    }

    /// Creates a new client core with specific search API version
    pub fn with_search_api_version<H>(
        host: H,
        credentials: Credentials,
        search_api_version: SearchApiVersion,
    ) -> Result<Self>
    where
        H: Into<String>,
    {
        match Url::parse(&host.into()) {
            Ok(host) => Ok(ClientCore {
                host,
                credentials,
                search_api_version,
                #[cfg(feature = "cache")]
                cache: Arc::new(MemoryCache::new(std::time::Duration::from_secs(300))),
                #[cfg(feature = "cache")]
                cache_config: RuntimeCacheConfig::default(),
            }),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Creates a new client core with custom cache configuration
    #[cfg(feature = "cache")]
    pub fn with_cache<H>(
        host: H,
        credentials: Credentials,
        cache: Arc<dyn Cache>,
        cache_config: RuntimeCacheConfig,
    ) -> Result<Self>
    where
        H: Into<String>,
    {
        match Url::parse(&host.into()) {
            Ok(host) => Ok(ClientCore {
                host,
                credentials,
                search_api_version: SearchApiVersion::default(),
                cache,
                cache_config,
            }),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Builds the API URL for a request
    pub fn build_url(&self, api_name: &str, endpoint: &str) -> Result<Url> {
        self.host
            .join(&format!("rest/{api_name}/latest{endpoint}"))
            .map_err(Error::from)
    }

    /// Builds the API URL for a request with specific version
    pub fn build_versioned_url(
        &self,
        api_name: &str,
        version: Option<&str>,
        endpoint: &str,
    ) -> Result<Url> {
        let version_part = version.unwrap_or("latest");
        self.host
            .join(&format!("rest/{api_name}/{version_part}{endpoint}"))
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

    /// Check cache for a response before making a request
    #[cfg(feature = "cache")]
    pub fn check_cache<D>(&self, method: &str, endpoint: &str) -> Option<D>
    where
        D: DeserializeOwned,
    {
        // Only cache GET requests
        if method != "GET" || !self.cache_config.should_cache_endpoint(endpoint) {
            return None;
        }

        let cache_key = generate_cache_key(endpoint, "");
        if let Some(cached_data) = self.cache.get(&cache_key) {
            // The cached data is raw JSON bytes
            match std::str::from_utf8(&cached_data) {
                Ok(json_str) => {
                    let processed_json = if json_str.is_empty() {
                        "null"
                    } else {
                        json_str
                    };
                    match serde_json::from_str::<D>(processed_json) {
                        Ok(result) => {
                            debug!(cache_key = %cache_key, "Cache hit");
                            Some(result)
                        }
                        Err(e) => {
                            debug!(cache_key = %cache_key, error = %e, "Failed to deserialize cached JSON");
                            None
                        }
                    }
                }
                Err(e) => {
                    debug!(cache_key = %cache_key, error = %e, "Cached data is not valid UTF-8");
                    None
                }
            }
        } else {
            None
        }
    }

    /// Store a raw JSON response in cache after a successful request
    #[cfg(feature = "cache")]
    pub fn store_raw_response(&self, method: &str, endpoint: &str, raw_json: &str) {
        // Only cache GET requests
        if method != "GET" || !self.cache_config.should_cache_endpoint(endpoint) {
            return;
        }

        let cache_key = generate_cache_key(endpoint, "");
        let strategy = self.cache_config.strategy_for_endpoint(endpoint);

        self.cache
            .set(&cache_key, raw_json.as_bytes().to_vec(), strategy.ttl);
        debug!(
            cache_key = %cache_key,
            ttl_secs = strategy.ttl.as_secs(),
            size_bytes = raw_json.len(),
            "Raw JSON response cached"
        );
    }

    /// Clear all cached responses
    #[cfg(feature = "cache")]
    pub fn clear_cache(&self) {
        self.cache.clear();
        debug!("Cache cleared");
    }

    /// Get cache statistics
    #[cfg(feature = "cache")]
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats()
    }

    /// Generate OAuth 1.0a authorization header if using OAuth credentials
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `url` - Full request URL
    ///
    /// # Returns
    ///
    /// Optional OAuth authorization header value
    #[cfg(feature = "oauth")]
    pub fn get_oauth_header(&self, method: &str, url: &str) -> Result<Option<String>> {
        match &self.credentials {
            Credentials::OAuth1a {
                consumer_key,
                private_key_pem,
                access_token,
                access_token_secret,
            } => {
                let header = crate::oauth::generate_oauth_header(
                    method,
                    url,
                    consumer_key,
                    private_key_pem,
                    access_token,
                    access_token_secret,
                )?;
                Ok(Some(header))
            }
            _ => Ok(None),
        }
    }

    /// Apply credentials to a sync request builder
    ///
    /// Note: For OAuth 1.0a, the Authorization header must be set separately using
    /// `get_oauth_header()` before creating the request builder, as signature generation
    /// requires knowledge of the HTTP method and URL.
    pub fn apply_credentials_sync(
        &self,
        builder: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match &self.credentials {
            Credentials::Anonymous => builder,
            Credentials::Basic(user, pass) => {
                builder.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(token) => builder.bearer_auth(token.to_owned()),
            Credentials::Cookie(jsessionid) => builder.header(
                reqwest::header::COOKIE,
                format!("JSESSIONID={}", jsessionid),
            ),
            #[cfg(feature = "oauth")]
            Credentials::OAuth1a { .. } => {
                // OAuth header is applied separately via get_oauth_header()
                // because it needs method and URL information
                builder
            }
        }
    }

    /// Apply credentials to an async request builder
    ///
    /// Note: For OAuth 1.0a, the Authorization header must be set separately using
    /// `get_oauth_header()` before creating the request builder, as signature generation
    /// requires knowledge of the HTTP method and URL.
    pub fn apply_credentials_async(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        match &self.credentials {
            Credentials::Anonymous => builder,
            Credentials::Basic(user, pass) => {
                builder.basic_auth(user.to_owned(), Some(pass.to_owned()))
            }
            Credentials::Bearer(token) => builder.bearer_auth(token.to_owned()),
            Credentials::Cookie(jsessionid) => builder.header(
                reqwest::header::COOKIE,
                format!("JSESSIONID={}", jsessionid),
            ),
            #[cfg(feature = "oauth")]
            Credentials::OAuth1a { .. } => {
                // OAuth header is applied separately via get_oauth_header()
                // because it needs method and URL information
                builder
            }
        }
    }

    /// Detect Jira deployment type based on host URL
    pub fn detect_deployment_type(&self) -> JiraDeploymentType {
        let host_str = self.host.host_str().unwrap_or("");
        if host_str.contains(".atlassian.net") {
            JiraDeploymentType::Cloud
        } else {
            // For self-hosted instances, we can't easily distinguish between
            // Server and Data Center from the URL alone, so we use Unknown
            // which will trigger capability testing
            JiraDeploymentType::Unknown
        }
    }

    /// Get the optimal search API version based on deployment type and configuration
    pub fn get_search_api_version(&self) -> SearchApiVersion {
        match &self.search_api_version {
            SearchApiVersion::Auto => {
                match self.detect_deployment_type() {
                    JiraDeploymentType::Cloud => SearchApiVersion::V3,
                    // For self-hosted, we'll need runtime detection
                    JiraDeploymentType::DataCenter
                    | JiraDeploymentType::Server
                    | JiraDeploymentType::Unknown => SearchApiVersion::V2, // Default to V2 for now
                }
            }
            version => version.clone(),
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
