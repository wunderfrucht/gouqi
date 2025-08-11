use std::io::Read;
use tracing::debug;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Method, blocking::Client};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::attachments::Attachments;
use crate::boards::Boards;
use crate::components::Components;
use crate::core::ClientCore;
use crate::issues::Issues;
use crate::rep::Session;
use crate::search::Search;
use crate::sprints::Sprints;
use crate::transitions::Transitions;
use crate::versions::Versions;
use crate::{Credentials, Result};

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
        let url = self.core.build_url(api_name, endpoint)?;
        debug!("url -> {:?}", url);

        let mut req = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json");

        req = self.core.apply_credentials_sync(req);

        if let Some(body) = body {
            req = req.body(body);
        }
        debug!("req '{:?}'", req);

        let mut res = req.send()?;

        let mut body = String::new();
        res.read_to_string(&mut body)?;
        debug!("status {:?} body '{:?}'", res.status(), body);

        self.core.process_response(res.status(), &body)
    }
}
