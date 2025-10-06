//! Interfaces for accessing and managing users
//!
//! This module provides methods to search for users, get user details,
//! and find users assignable to projects and issues.

use url::form_urlencoded;

use crate::{Jira, Result, User};

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

/// User management interface
#[derive(Debug)]
pub struct Users {
    jira: Jira,
}

/// Options for searching users
#[derive(Debug, Default, Clone)]
pub struct UserSearchOptions {
    pub query: Option<String>,
    pub start_at: Option<u64>,
    pub max_results: Option<u64>,
}

impl UserSearchOptions {
    pub fn builder() -> UserSearchOptionsBuilder {
        UserSearchOptionsBuilder::default()
    }

    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(("query".to_string(), query.clone()));
        }
        if let Some(start_at) = self.start_at {
            params.push(("startAt".to_string(), start_at.to_string()));
        }
        if let Some(max_results) = self.max_results {
            params.push(("maxResults".to_string(), max_results.to_string()));
        }

        params
    }
}

/// Builder for UserSearchOptions
#[derive(Debug, Default)]
pub struct UserSearchOptionsBuilder {
    query: Option<String>,
    start_at: Option<u64>,
    max_results: Option<u64>,
}

impl UserSearchOptionsBuilder {
    pub fn query<S: Into<String>>(mut self, query: S) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn start_at(mut self, start_at: u64) -> Self {
        self.start_at = Some(start_at);
        self
    }

    pub fn max_results(mut self, max_results: u64) -> Self {
        self.max_results = Some(max_results);
        self
    }

    pub fn build(self) -> UserSearchOptions {
        UserSearchOptions {
            query: self.query,
            start_at: self.start_at,
            max_results: self.max_results,
        }
    }
}

/// Options for finding assignable users
#[derive(Debug, Default, Clone)]
pub struct AssignableUserOptions {
    pub query: Option<String>,
    pub start_at: Option<u64>,
    pub max_results: Option<u64>,
}

impl AssignableUserOptions {
    pub fn builder() -> AssignableUserOptionsBuilder {
        AssignableUserOptionsBuilder::default()
    }

    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(("query".to_string(), query.clone()));
        }
        if let Some(start_at) = self.start_at {
            params.push(("startAt".to_string(), start_at.to_string()));
        }
        if let Some(max_results) = self.max_results {
            params.push(("maxResults".to_string(), max_results.to_string()));
        }

        params
    }
}

/// Builder for AssignableUserOptions
#[derive(Debug, Default)]
pub struct AssignableUserOptionsBuilder {
    query: Option<String>,
    start_at: Option<u64>,
    max_results: Option<u64>,
}

impl AssignableUserOptionsBuilder {
    pub fn query<S: Into<String>>(mut self, query: S) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn start_at(mut self, start_at: u64) -> Self {
        self.start_at = Some(start_at);
        self
    }

    pub fn max_results(mut self, max_results: u64) -> Self {
        self.max_results = Some(max_results);
        self
    }

    pub fn build(self) -> AssignableUserOptions {
        AssignableUserOptions {
            query: self.query,
            start_at: self.start_at,
            max_results: self.max_results,
        }
    }
}

impl Users {
    pub fn new(jira: &Jira) -> Users {
        Users { jira: jira.clone() }
    }

    /// Get user by account ID
    ///
    /// Returns details of a single user identified by their account ID.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let user = jira.users().get("5b10a2844c20165700ede21g").unwrap();
    /// println!("User: {}", user.display_name);
    /// ```
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-users/#api-rest-api-3-user-get)
    /// for more information
    pub fn get<I>(&self, account_id: I) -> Result<User>
    where
        I: Into<String>,
    {
        let account_id = account_id.into();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("accountId", &account_id)
            .finish();
        self.jira.get("api", &format!("/user?{}", query))
    }

    /// Search for users
    ///
    /// Returns a list of users matching the search query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::users::UserSearchOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = UserSearchOptions::builder()
    ///     .query("john")
    ///     .max_results(10)
    ///     .build();
    /// let users = jira.users().search(&options).unwrap();
    /// ```
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-search-get)
    /// for more information
    pub fn search(&self, options: &UserSearchOptions) -> Result<Vec<User>> {
        let params = options.to_query_params();
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        let path = if query.is_empty() {
            "/user/search".to_string()
        } else {
            format!("/user/search?{}", query)
        };

        self.jira.get("api", &path)
    }

    /// Get users assignable to a project
    ///
    /// Returns users who can be assigned to issues in the specified project.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::users::AssignableUserOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = AssignableUserOptions::builder()
    ///     .max_results(50)
    ///     .build();
    /// let users = jira.users().get_assignable_users("PROJ", &options).unwrap();
    /// ```
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-assignable-search-get)
    /// for more information
    pub fn get_assignable_users<P>(
        &self,
        project: P,
        options: &AssignableUserOptions,
    ) -> Result<Vec<User>>
    where
        P: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("project".to_string(), project.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .get("api", &format!("/user/assignable/search?{}", query))
    }

    /// Get users assignable to an issue
    ///
    /// Returns users who can be assigned to the specified issue.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::users::AssignableUserOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = AssignableUserOptions::default();
    /// let users = jira.users().get_assignable_users_for_issue("PROJ-123", &options).unwrap();
    /// ```
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-assignable-search-get)
    /// for more information
    pub fn get_assignable_users_for_issue<I>(
        &self,
        issue_key: I,
        options: &AssignableUserOptions,
    ) -> Result<Vec<User>>
    where
        I: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("issueKey".to_string(), issue_key.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .get("api", &format!("/user/assignable/search?{}", query))
    }
}

#[cfg(feature = "async")]
/// Async version of the Users interface
#[derive(Debug)]
pub struct AsyncUsers {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncUsers {
    pub fn new(jira: &AsyncJira) -> AsyncUsers {
        AsyncUsers { jira: jira.clone() }
    }

    /// Get user by account ID (async)
    ///
    /// Returns details of a single user identified by their account ID.
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-users/#api-rest-api-3-user-get)
    /// for more information
    pub async fn get<I>(&self, account_id: I) -> Result<User>
    where
        I: Into<String>,
    {
        let account_id = account_id.into();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("accountId", &account_id)
            .finish();
        self.jira.get("api", &format!("/user?{}", query)).await
    }

    /// Search for users (async)
    ///
    /// Returns a list of users matching the search query.
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-search-get)
    /// for more information
    pub async fn search(&self, options: &UserSearchOptions) -> Result<Vec<User>> {
        let params = options.to_query_params();
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        let path = if query.is_empty() {
            "/user/search".to_string()
        } else {
            format!("/user/search?{}", query)
        };

        self.jira.get("api", &path).await
    }

    /// Get users assignable to a project (async)
    ///
    /// Returns users who can be assigned to issues in the specified project.
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-assignable-search-get)
    /// for more information
    pub async fn get_assignable_users<P>(
        &self,
        project: P,
        options: &AssignableUserOptions,
    ) -> Result<Vec<User>>
    where
        P: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("project".to_string(), project.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .get("api", &format!("/user/assignable/search?{}", query))
            .await
    }

    /// Get users assignable to an issue (async)
    ///
    /// Returns users who can be assigned to the specified issue.
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-assignable-search-get)
    /// for more information
    pub async fn get_assignable_users_for_issue<I>(
        &self,
        issue_key: I,
        options: &AssignableUserOptions,
    ) -> Result<Vec<User>>
    where
        I: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("issueKey".to_string(), issue_key.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .get("api", &format!("/user/assignable/search?{}", query))
            .await
    }
}
