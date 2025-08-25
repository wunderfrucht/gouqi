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
#[derive(Clone)]
pub struct ClientCore {
    pub host: Url,
    pub credentials: Credentials,
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
        match Url::parse(&host.into()) {
            Ok(host) => Ok(ClientCore {
                host,
                credentials,
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

    /// Apply credentials to a sync request builder
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
        }
    }

    /// Apply credentials to an async request builder
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
