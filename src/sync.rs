use std::io::Read;
use tracing::debug;

use reqwest::header::CONTENT_TYPE;
use reqwest::{blocking::Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::attachments::Attachments;
use crate::boards::Boards;
use crate::components::Components;
use crate::issues::Issues;
use crate::rep::Session;
use crate::search::Search;
use crate::sprints::Sprints;
use crate::transitions::Transitions;
use crate::versions::Versions;
use crate::{Credentials, Error, Errors, Result};

/// Entrypoint into client interface
/// <https://docs.atlassian.com/jira/REST/latest/>
#[derive(Clone, Debug)]
pub struct Jira {
    pub(crate) host: Url,
    pub(crate) credentials: Credentials,
    client: Client,
}

impl Jira {
    /// Creates a new instance of a jira client
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

    /// Creates a new instance of a jira client using a specified reqwest client
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

    // Return issues interface
    #[tracing::instrument]
    pub fn issues(&self) -> Issues {
        Issues::new(self)
    }

    // Return attachments interface
    pub fn attachments(&self) -> Attachments {
        Attachments::new(self)
    }

    // Return components interface
    pub fn components(&self) -> Components {
        Components::new(self)
    }

    // Return boards interface
    #[tracing::instrument]
    pub fn boards(&self) -> Boards {
        Boards::new(self)
    }

    // Return boards interface
    #[tracing::instrument]
    pub fn sprints(&self) -> Sprints {
        Sprints::new(self)
    }

    #[tracing::instrument]
    pub fn versions(&self) -> Versions {
        Versions::new(self)
    }

    pub fn session(&self) -> Result<Session> {
        self.get("auth", "/session")
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
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json POST request: {}", data);
        self.request::<D>(Method::POST, api_name, endpoint, Some(data.into_bytes()))
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
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json request: {}", data);
        self.request::<D>(Method::PUT, api_name, endpoint, Some(data.into_bytes()))
    }

    #[tracing::instrument]
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
        let url = self
            .host
            .join(&format!("rest/{api_name}/latest{endpoint}"))?;
        debug!("url -> {:?}", url);

        let mut req = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json");

        req = self.credentials.apply(req);

        if let Some(body) = body {
            req = req.body(body);
        }
        debug!("req '{:?}'", req);

        let mut res = req.send()?;

        let mut body = String::new();
        res.read_to_string(&mut body)?;
        debug!("status {:?} body '{:?}'", res.status(), body);
        match res.status() {
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::METHOD_NOT_ALLOWED => Err(Error::MethodNotAllowed),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            client_err if client_err.is_client_error() => Err(Error::Fault {
                code: res.status(),
                errors: serde_json::from_str::<Errors>(&body)?,
            }),
            _ => {
                let data = if body.is_empty() { "null" } else { &body };
                Ok(serde_json::from_str::<D>(data)?)
            }
        }
    }
}
