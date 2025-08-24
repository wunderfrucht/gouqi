// Third party
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use url::Url;
use url::form_urlencoded;

// Ours
use crate::{
    Credentials, Error, GouqiConfig, Result,
    env::{load_config_from_env, load_credentials_from_env, load_host_from_env},
};

/// Options availble for search
#[derive(Default, Clone, Debug)]
pub struct SearchOptions {
    params: HashMap<&'static str, String>,
}

impl SearchOptions {
    /// Return a new instance of a builder for options
    pub fn builder() -> SearchOptionsBuilder {
        SearchOptionsBuilder::new()
    }

    /// Serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }

    pub fn as_builder(&self) -> SearchOptionsBuilder {
        SearchOptionsBuilder::copy_from(self)
    }
}

/// A builder interface for search option. Typically this
/// is initialized with SearchOptions::builder()
#[derive(Default, Debug)]
pub struct SearchOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl SearchOptionsBuilder {
    pub fn new() -> SearchOptionsBuilder {
        SearchOptionsBuilder {
            ..Default::default()
        }
    }

    fn copy_from(search_options: &SearchOptions) -> SearchOptionsBuilder {
        SearchOptionsBuilder {
            params: search_options.params.clone(),
        }
    }

    pub fn fields<F>(&mut self, fs: Vec<F>) -> &mut SearchOptionsBuilder
    where
        F: Into<String>,
    {
        self.params.insert(
            "fields",
            fs.into_iter()
                .map(|f| f.into())
                .collect::<Vec<String>>()
                .join(","),
        );
        self
    }

    pub fn validate(&mut self, v: bool) -> &mut SearchOptionsBuilder {
        self.params.insert("validateQuery", v.to_string());
        self
    }

    pub fn max_results(&mut self, m: u64) -> &mut SearchOptionsBuilder {
        self.params.insert("maxResults", m.to_string());
        self
    }

    pub fn start_at(&mut self, s: u64) -> &mut SearchOptionsBuilder {
        self.params.insert("startAt", s.to_string());
        self
    }

    pub fn type_name(&mut self, t: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("type", t.to_string());
        self
    }

    pub fn name(&mut self, n: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("name", n.to_string());
        self
    }

    pub fn project_key_or_id(&mut self, id: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("projectKeyOrId", id.to_string());
        self
    }

    pub fn expand<E>(&mut self, ex: Vec<E>) -> &mut SearchOptionsBuilder
    where
        E: Into<String>,
    {
        self.params.insert(
            "expand",
            ex.into_iter()
                .map(|e| e.into())
                .collect::<Vec<String>>()
                .join(","),
        );
        self
    }

    pub fn state(&mut self, s: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("state", s.to_string());
        self
    }

    pub fn jql(&mut self, s: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("jql", s.to_string());
        self
    }

    pub fn validate_query(&mut self, v: bool) -> &mut SearchOptionsBuilder {
        self.params.insert("validateQuery", v.to_string());
        self
    }

    pub fn build(&self) -> SearchOptions {
        SearchOptions {
            params: self.params.clone(),
        }
    }
}

/// Enhanced builder for Jira client configuration
///
/// This builder provides a fluent interface for configuring all aspects of the Jira client,
/// including authentication, timeouts, connection pools, caching, metrics, and custom fields.
///
/// # Examples
///
/// ```rust,no_run
/// use gouqi::{JiraBuilder, Credentials, FieldSchema};
/// use std::time::Duration;
///
/// // Basic usage
/// let jira = JiraBuilder::new()
///     .host("https://company.atlassian.net")
///     .credentials(Credentials::Basic("user".to_string(), "token".to_string()))
///     .timeout(Duration::from_secs(60))
///     .build_with_validation()?;
///
/// // Advanced configuration
/// let jira = JiraBuilder::new()
///     .host("https://company.atlassian.net")
///     .credentials(Credentials::Bearer("token".to_string()))
///     .config_from_file("config.yaml")?
///     .custom_field("story_points", FieldSchema::number(false, Some(0.0), Some(100.0)))
///     .memory_cache(Duration::from_secs(300), 1000)
///     .retry_policy(3, Duration::from_millis(500), Duration::from_secs(10))
///     .build_with_validation()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct JiraBuilder {
    host: Option<String>,
    credentials: Option<Credentials>,
    config: GouqiConfig,
    custom_fields: HashMap<String, FieldSchema>,
    validate_ssl: bool,
    user_agent: Option<String>,
}

impl JiraBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            host: None,
            credentials: None,
            config: GouqiConfig::default(),
            custom_fields: HashMap::new(),
            validate_ssl: true,
            user_agent: None,
        }
    }

    /// Set Jira host URL
    ///
    /// # Panics
    ///
    /// This function will panic if the URL is invalid
    pub fn host<H: Into<String>>(mut self, host: H) -> Self {
        let host_str = host.into();

        // Validate URL format
        if Url::parse(&host_str).is_err() {
            panic!("Invalid host URL: {}", host_str);
        }

        self.host = Some(host_str);
        self
    }

    /// Set authentication credentials
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Load configuration from file (JSON, YAML, or TOML)
    ///
    /// # Panics
    ///
    /// This function will panic if the config file cannot be read or parsed
    pub fn config_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let config = GouqiConfig::from_file(path)?;
        self.config = self.config.merge(config);
        Ok(self)
    }

    /// Load configuration from environment variables
    ///
    /// This will load host, credentials, and various configuration options from
    /// environment variables with the `JIRA_` prefix.
    pub fn config_from_env(mut self) -> Result<Self> {
        // Load host if not already set
        if self.host.is_none() {
            if let Some(host) = load_host_from_env() {
                self = self.host(host);
            }
        }

        // Load credentials if not already set
        if self.credentials.is_none() {
            let creds = load_credentials_from_env();
            if !matches!(creds, Credentials::Anonymous) {
                self = self.credentials(creds);
            }
        }

        // Merge environment configuration
        let env_config = load_config_from_env();
        self.config = self.config.merge(env_config);

        Ok(self)
    }

    /// Apply a predefined configuration template
    pub fn config_template(mut self, template: ConfigTemplate) -> Self {
        self.config = match template {
            ConfigTemplate::Default => GouqiConfig::default(),
            ConfigTemplate::HighThroughput => GouqiConfig::high_throughput(),
            ConfigTemplate::LowResource => GouqiConfig::low_resource(),
        };
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout.default = timeout;
        self
    }

    /// Set connection timeout  
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout.connect = timeout;
        self
    }

    /// Set read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout.read = timeout;
        self
    }

    /// Configure retry policy
    pub fn retry_policy(
        mut self,
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.config.retry.max_attempts = max_attempts;
        self.config.retry.base_delay = base_delay;
        self.config.retry.max_delay = max_delay;
        self
    }

    /// Set retry backoff multiplier
    pub fn retry_backoff(mut self, multiplier: f64) -> Self {
        self.config.retry.backoff_multiplier = multiplier;
        self
    }

    /// Set which HTTP status codes should trigger retries
    pub fn retry_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.config.retry.retry_status_codes = codes;
        self
    }

    /// Set connection pool size
    pub fn connection_pool_size(mut self, size: usize) -> Self {
        self.config.connection_pool.max_connections_per_host = size;
        self
    }

    /// Configure connection pool settings
    pub fn connection_pool(
        mut self,
        max_connections: usize,
        idle_timeout: Duration,
        http2: bool,
    ) -> Self {
        self.config.connection_pool.max_connections_per_host = max_connections;
        self.config.connection_pool.idle_timeout = idle_timeout;
        self.config.connection_pool.http2 = http2;
        self
    }

    /// Enable or disable SSL certificate validation
    pub fn validate_ssl(mut self, validate: bool) -> Self {
        self.validate_ssl = validate;
        self
    }

    /// Set custom User-Agent header
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Enable caching with default settings
    pub fn enable_cache(mut self) -> Self {
        self.config.cache.enabled = true;
        self
    }

    /// Disable caching
    pub fn disable_cache(mut self) -> Self {
        self.config.cache.enabled = false;
        self
    }

    /// Configure memory cache with custom settings
    pub fn memory_cache(mut self, default_ttl: Duration, max_entries: usize) -> Self {
        self.config.cache.enabled = true;
        self.config.cache.default_ttl = default_ttl;
        self.config.cache.max_entries = max_entries;
        self
    }

    /// Configure rate limiting
    pub fn rate_limit(mut self, requests_per_second: f64, burst_capacity: u32) -> Self {
        self.config.rate_limiting.enabled = true;
        self.config.rate_limiting.requests_per_second = requests_per_second;
        self.config.rate_limiting.burst_capacity = burst_capacity;
        self
    }

    /// Disable rate limiting
    pub fn disable_rate_limiting(mut self) -> Self {
        self.config.rate_limiting.enabled = false;
        self
    }

    /// Enable metrics collection
    pub fn enable_metrics(mut self) -> Self {
        self.config.metrics.enabled = true;
        self
    }

    /// Disable metrics collection
    pub fn disable_metrics(mut self) -> Self {
        self.config.metrics.enabled = false;
        self
    }

    /// Configure metrics collection settings
    pub fn metrics_config(mut self, collection_interval: Duration, export_format: &str) -> Self {
        self.config.metrics.enabled = true;
        self.config.metrics.collection_interval = collection_interval;
        self.config.metrics.export.format = export_format.to_string();
        self
    }

    /// Add custom field schema
    ///
    /// # Panics
    ///
    /// This function will panic if the field name is empty
    pub fn custom_field<N: Into<String>>(mut self, name: N, schema: FieldSchema) -> Self {
        let field_name = name.into();
        assert!(!field_name.is_empty(), "Field name cannot be empty");

        self.custom_fields.insert(field_name, schema);
        self
    }

    /// Add multiple custom fields
    pub fn custom_fields(mut self, fields: HashMap<String, FieldSchema>) -> Self {
        self.custom_fields.extend(fields);
        self
    }

    /// Build and validate the configuration
    ///
    /// This method performs comprehensive validation of all configuration settings
    /// and returns an error if any issues are found.
    ///
    /// # Panics
    ///
    /// This function will panic if the configuration is invalid
    pub fn build_with_validation(self) -> Result<crate::Jira> {
        // Validate required fields - use clone to avoid move
        let host = self.host.clone().ok_or_else(|| Error::ConfigError {
            message: "Host URL is required".to_string(),
        })?;

        let credentials = self.credentials.clone().ok_or_else(|| Error::ConfigError {
            message: "Credentials are required".to_string(),
        })?;

        // Validate host URL
        let _parsed_url = Url::parse(&host).map_err(|e| Error::ConfigError {
            message: format!("Invalid host URL '{}': {}", host, e),
        })?;

        // Validate configuration
        self.config.validate()?;

        // Validate credentials
        self.validate_credentials(&credentials)?;

        // Validate custom fields
        self.validate_custom_fields()?;

        // Build the client
        self.build()
    }

    /// Build the Jira client (without validation)
    ///
    /// This method builds the client without performing validation.
    /// Use `build_with_validation()` for production code.
    pub fn build(self) -> Result<crate::Jira> {
        let host = self
            .host
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        let credentials = self.credentials.unwrap_or(Credentials::Anonymous);

        // Create the basic client
        let client = crate::Jira::new(host, credentials)?;

        // TODO: Apply advanced configuration to the client
        // This would require extending the core client to support these features

        Ok(client)
    }

    /// Validate credentials
    fn validate_credentials(&self, credentials: &Credentials) -> Result<()> {
        match credentials {
            Credentials::Basic(user, pass) => {
                if user.is_empty() || pass.is_empty() {
                    return Err(Error::ConfigError {
                        message: "Username and password cannot be empty for Basic auth".to_string(),
                    });
                }
            }
            Credentials::Bearer(token) => {
                if token.is_empty() {
                    return Err(Error::ConfigError {
                        message: "Bearer token cannot be empty".to_string(),
                    });
                }
            }
            Credentials::Cookie(cookie) => {
                if cookie.is_empty() {
                    return Err(Error::ConfigError {
                        message: "Cookie cannot be empty".to_string(),
                    });
                }
            }
            Credentials::Anonymous => {
                // Anonymous is always valid
            }
        }
        Ok(())
    }

    /// Validate custom field schemas
    fn validate_custom_fields(&self) -> Result<()> {
        for (field_name, schema) in &self.custom_fields {
            schema.validate(field_name)?;
        }
        Ok(())
    }

    /// Getter methods for testing
    #[cfg(test)]
    pub fn get_host(&self) -> &Option<String> {
        &self.host
    }

    #[cfg(test)]
    pub fn get_credentials(&self) -> &Option<Credentials> {
        &self.credentials
    }

    #[cfg(test)]
    pub fn get_config(&self) -> &GouqiConfig {
        &self.config
    }

    #[cfg(test)]
    pub fn get_custom_fields(&self) -> &HashMap<String, FieldSchema> {
        &self.custom_fields
    }
}

impl Default for JiraBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined configuration templates
#[derive(Debug, Clone, Copy)]
pub enum ConfigTemplate {
    /// Default balanced configuration
    Default,
    /// Optimized for high-throughput scenarios
    HighThroughput,
    /// Optimized for low-resource environments
    LowResource,
}

/// Custom field schema definition for Jira fields
///
/// This allows you to define validation rules and metadata for custom fields
/// that your application uses with Jira.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// The type of field (string, number, enum, etc.)
    pub field_type: String,
    /// Whether this field is required
    pub required: bool,
    /// Default value for the field
    pub default_value: Option<serde_json::Value>,
    /// Allowed values (for enum-type fields)
    pub allowed_values: Option<Vec<serde_json::Value>>,
    /// Custom properties for validation
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl FieldSchema {
    /// Create a simple text field schema
    pub fn text(required: bool) -> Self {
        Self {
            field_type: "string".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: HashMap::new(),
        }
    }

    /// Create a number field schema with optional min/max validation
    ///
    /// # Panics
    ///
    /// This function will panic if min is greater than max
    pub fn number(required: bool, min: Option<f64>, max: Option<f64>) -> Self {
        let mut properties = HashMap::new();

        if let (Some(min_val), Some(max_val)) = (min, max) {
            assert!(
                min_val <= max_val,
                "Minimum value cannot be greater than maximum value"
            );
        }

        if let Some(min_val) = min {
            properties.insert(
                "minimum".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(min_val).unwrap()),
            );
        }
        if let Some(max_val) = max {
            properties.insert(
                "maximum".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(max_val).unwrap()),
            );
        }

        Self {
            field_type: "number".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: properties,
        }
    }

    /// Create an integer field schema with optional min/max validation
    ///
    /// # Panics
    ///
    /// This function will panic if min is greater than max
    pub fn integer(required: bool, min: Option<i64>, max: Option<i64>) -> Self {
        let mut properties = HashMap::new();

        if let (Some(min_val), Some(max_val)) = (min, max) {
            assert!(
                min_val <= max_val,
                "Minimum value cannot be greater than maximum value"
            );
        }

        if let Some(min_val) = min {
            properties.insert(
                "minimum".to_string(),
                serde_json::Value::Number(serde_json::Number::from(min_val)),
            );
        }
        if let Some(max_val) = max {
            properties.insert(
                "maximum".to_string(),
                serde_json::Value::Number(serde_json::Number::from(max_val)),
            );
        }

        Self {
            field_type: "integer".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: properties,
        }
    }

    /// Create a boolean field schema
    pub fn boolean(required: bool, default: Option<bool>) -> Self {
        Self {
            field_type: "boolean".to_string(),
            required,
            default_value: default.map(serde_json::Value::Bool),
            allowed_values: None,
            custom_properties: HashMap::new(),
        }
    }

    /// Create an enum field schema with allowed values
    pub fn enumeration<V: Serialize>(required: bool, allowed_values: Vec<V>) -> Result<Self> {
        let values: std::result::Result<Vec<serde_json::Value>, crate::Error> = allowed_values
            .into_iter()
            .map(|v| serde_json::to_value(v).map_err(Error::Serde))
            .collect();

        Ok(Self {
            field_type: "enum".to_string(),
            required,
            default_value: None,
            allowed_values: Some(values?),
            custom_properties: HashMap::new(),
        })
    }

    /// Create a date field schema
    pub fn date(required: bool) -> Self {
        Self {
            field_type: "date".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: HashMap::new(),
        }
    }

    /// Create a datetime field schema
    pub fn datetime(required: bool) -> Self {
        Self {
            field_type: "datetime".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: HashMap::new(),
        }
    }

    /// Create an array field schema
    pub fn array(required: bool, item_type: &str) -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "item_type".to_string(),
            serde_json::Value::String(item_type.to_string()),
        );

        Self {
            field_type: "array".to_string(),
            required,
            default_value: None,
            allowed_values: None,
            custom_properties: properties,
        }
    }

    /// Set a default value for the field
    pub fn with_default<V: Serialize>(mut self, default: V) -> Result<Self> {
        self.default_value = Some(serde_json::to_value(default).map_err(Error::Serde)?);
        Ok(self)
    }

    /// Add a custom property to the schema
    pub fn with_property<V: Serialize>(mut self, key: &str, value: V) -> Result<Self> {
        self.custom_properties.insert(
            key.to_string(),
            serde_json::to_value(value).map_err(Error::Serde)?,
        );
        Ok(self)
    }

    /// Validate the field schema
    fn validate(&self, field_name: &str) -> Result<()> {
        if self.field_type.is_empty() {
            return Err(Error::FieldSchemaError {
                field: field_name.to_string(),
                message: "Field type cannot be empty".to_string(),
            });
        }

        // Validate enum fields have allowed values
        if self.field_type == "enum" && self.allowed_values.is_none() {
            return Err(Error::FieldSchemaError {
                field: field_name.to_string(),
                message: "Enum fields must specify allowed values".to_string(),
            });
        }

        // Validate number/integer ranges
        if matches!(self.field_type.as_str(), "number" | "integer") {
            if let (Some(min), Some(max)) = (
                self.custom_properties
                    .get("minimum")
                    .and_then(|v| v.as_f64()),
                self.custom_properties
                    .get("maximum")
                    .and_then(|v| v.as_f64()),
            ) {
                if min > max {
                    return Err(Error::FieldSchemaError {
                        field: field_name.to_string(),
                        message: "Minimum value cannot be greater than maximum value".to_string(),
                    });
                }
            }
        }

        // Validate array item types
        if self.field_type == "array" && !self.custom_properties.contains_key("item_type") {
            return Err(Error::FieldSchemaError {
                field: field_name.to_string(),
                message: "Array fields must specify item_type".to_string(),
            });
        }

        Ok(())
    }
}
