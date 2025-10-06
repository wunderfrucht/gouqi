//! Interfaces for accessing and managing groups
//!
//! This module provides methods to list groups, manage group membership,
//! and perform group CRUD operations.

use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{Jira, Result, User};

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

/// Group management interface
#[derive(Debug)]
pub struct Groups {
    jira: Jira,
}

/// Group information returned by the picker endpoint
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GroupPickerResult {
    pub name: String,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    pub html: String,
}

/// Response from groups picker endpoint
#[derive(Deserialize, Debug)]
pub struct GroupsPickerResponse {
    pub header: String,
    pub total: u64,
    pub groups: Vec<GroupPickerResult>,
}

/// Response from group members endpoint
#[derive(Deserialize, Debug)]
pub struct GroupMembersResponse {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    #[serde(rename = "startAt")]
    pub start_at: u32,
    pub total: u32,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub values: Vec<User>,
}

/// Request body for creating a group
#[derive(Serialize, Debug)]
pub struct CreateGroup {
    pub name: String,
}

/// Request body for adding a user to a group
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToGroup {
    pub account_id: String,
}

/// Options for listing groups
#[derive(Debug, Default, Clone)]
pub struct GroupSearchOptions {
    pub query: Option<String>,
    pub max_results: Option<u64>,
}

impl GroupSearchOptions {
    pub fn builder() -> GroupSearchOptionsBuilder {
        GroupSearchOptionsBuilder::default()
    }

    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(("query".to_string(), query.clone()));
        }
        if let Some(max_results) = self.max_results {
            params.push(("maxResults".to_string(), max_results.to_string()));
        }

        params
    }
}

/// Builder for GroupSearchOptions
#[derive(Debug, Default)]
pub struct GroupSearchOptionsBuilder {
    query: Option<String>,
    max_results: Option<u64>,
}

impl GroupSearchOptionsBuilder {
    pub fn query<S: Into<String>>(mut self, query: S) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn max_results(mut self, max_results: u64) -> Self {
        self.max_results = Some(max_results);
        self
    }

    pub fn build(self) -> GroupSearchOptions {
        GroupSearchOptions {
            query: self.query,
            max_results: self.max_results,
        }
    }
}

/// Options for getting group members
#[derive(Debug, Default, Clone)]
pub struct GroupMemberOptions {
    pub include_inactive_users: Option<bool>,
    pub start_at: Option<u64>,
    pub max_results: Option<u64>,
}

impl GroupMemberOptions {
    pub fn builder() -> GroupMemberOptionsBuilder {
        GroupMemberOptionsBuilder::default()
    }

    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(include_inactive) = self.include_inactive_users {
            params.push((
                "includeInactiveUsers".to_string(),
                include_inactive.to_string(),
            ));
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

/// Builder for GroupMemberOptions
#[derive(Debug, Default)]
pub struct GroupMemberOptionsBuilder {
    include_inactive_users: Option<bool>,
    start_at: Option<u64>,
    max_results: Option<u64>,
}

impl GroupMemberOptionsBuilder {
    pub fn include_inactive_users(mut self, include: bool) -> Self {
        self.include_inactive_users = Some(include);
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

    pub fn build(self) -> GroupMemberOptions {
        GroupMemberOptions {
            include_inactive_users: self.include_inactive_users,
            start_at: self.start_at,
            max_results: self.max_results,
        }
    }
}

impl Groups {
    pub fn new(jira: &Jira) -> Groups {
        Groups { jira: jira.clone() }
    }

    /// List groups matching a query
    ///
    /// Returns groups for group picker suggestions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::groups::GroupSearchOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = GroupSearchOptions::builder()
    ///     .query("developers")
    ///     .max_results(10)
    ///     .build();
    /// let result = jira.groups().list(&options).unwrap();
    /// println!("Found {} groups", result.total);
    /// ```
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-groups-picker-get)
    /// for more information
    pub fn list(&self, options: &GroupSearchOptions) -> Result<GroupsPickerResponse> {
        let params = options.to_query_params();
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        let path = if query.is_empty() {
            "/groups/picker".to_string()
        } else {
            format!("/groups/picker?{}", query)
        };

        self.jira.get("api", &path)
    }

    /// Get members of a group
    ///
    /// Returns a paginated list of all users in the specified group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::groups::GroupMemberOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = GroupMemberOptions::builder()
    ///     .max_results(50)
    ///     .build();
    /// let members = jira.groups().get_members("jira-developers", &options).unwrap();
    /// println!("Group has {} members", members.total);
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if the group is not found or if the user lacks permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-member-get)
    /// for more information
    pub fn get_members<I>(
        &self,
        group_id: I,
        options: &GroupMemberOptions,
    ) -> Result<GroupMembersResponse>
    where
        I: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("groupId".to_string(), group_id.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira.get("api", &format!("/group/member?{}", query))
    }

    /// Create a new group
    ///
    /// Creates a new group with the specified name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Basic(
    ///     "admin@example.com".to_string(),
    ///     "api_token".to_string()
    /// )).unwrap();
    /// jira.groups().create("new-team").unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group name already exists
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-post)
    /// for more information
    pub fn create<S>(&self, name: S) -> Result<serde_json::Value>
    where
        S: Into<String>,
    {
        let group = CreateGroup { name: name.into() };
        self.jira.post("api", "/group", group)
    }

    /// Delete a group
    ///
    /// Deletes the specified group. Optionally transfers restrictions to another group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Basic(
    ///     "admin@example.com".to_string(),
    ///     "api_token".to_string()
    /// )).unwrap();
    /// jira.groups().delete("old-team-id", None).unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-delete)
    /// for more information
    pub fn delete<I>(&self, group_id: I, swap_group_id: Option<String>) -> Result<()>
    where
        I: Into<String>,
    {
        let mut params = vec![("groupId".to_string(), group_id.into())];
        if let Some(swap) = swap_group_id {
            params.push(("swapGroupId".to_string(), swap));
        }

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/group?{}", query))?;
        Ok(())
    }

    /// Add a user to a group
    ///
    /// Adds the specified user to the specified group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Basic(
    ///     "admin@example.com".to_string(),
    ///     "api_token".to_string()
    /// )).unwrap();
    /// jira.groups().add_user("team-id", "user-account-id").unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group or user does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-user-post)
    /// for more information
    pub fn add_user<G, U>(&self, group_id: G, account_id: U) -> Result<serde_json::Value>
    where
        G: Into<String>,
        U: Into<String>,
    {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("groupId", &group_id.into())
            .finish();

        let request = AddUserToGroup {
            account_id: account_id.into(),
        };

        self.jira
            .post("api", &format!("/group/user?{}", query), request)
    }

    /// Remove a user from a group
    ///
    /// Removes the specified user from the specified group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Basic(
    ///     "admin@example.com".to_string(),
    ///     "api_token".to_string()
    /// )).unwrap();
    /// jira.groups().remove_user("team-id", "user-account-id").unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group or user does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-user-delete)
    /// for more information
    pub fn remove_user<G, U>(&self, group_id: G, account_id: U) -> Result<()>
    where
        G: Into<String>,
        U: Into<String>,
    {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("groupId", &group_id.into())
            .append_pair("accountId", &account_id.into())
            .finish();

        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/group/user?{}", query))?;
        Ok(())
    }
}

#[cfg(feature = "async")]
/// Async version of the Groups interface
#[derive(Debug)]
pub struct AsyncGroups {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncGroups {
    pub fn new(jira: &AsyncJira) -> AsyncGroups {
        AsyncGroups { jira: jira.clone() }
    }

    /// List groups matching a query (async)
    ///
    /// Returns groups for group picker suggestions.
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-groups-picker-get)
    /// for more information
    pub async fn list(&self, options: &GroupSearchOptions) -> Result<GroupsPickerResponse> {
        let params = options.to_query_params();
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        let path = if query.is_empty() {
            "/groups/picker".to_string()
        } else {
            format!("/groups/picker?{}", query)
        };

        self.jira.get("api", &path).await
    }

    /// Get members of a group (async)
    ///
    /// Returns a paginated list of all users in the specified group.
    ///
    /// # Panics
    ///
    /// This function will panic if the group is not found or if the user lacks permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-member-get)
    /// for more information
    pub async fn get_members<I>(
        &self,
        group_id: I,
        options: &GroupMemberOptions,
    ) -> Result<GroupMembersResponse>
    where
        I: Into<String>,
    {
        let mut params = options.to_query_params();
        params.push(("groupId".to_string(), group_id.into()));

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .get("api", &format!("/group/member?{}", query))
            .await
    }

    /// Create a new group (async)
    ///
    /// Creates a new group with the specified name.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group name already exists
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-post)
    /// for more information
    pub async fn create<S>(&self, name: S) -> Result<serde_json::Value>
    where
        S: Into<String>,
    {
        let group = CreateGroup { name: name.into() };
        self.jira.post("api", "/group", group).await
    }

    /// Delete a group (async)
    ///
    /// Deletes the specified group. Optionally transfers restrictions to another group.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-delete)
    /// for more information
    pub async fn delete<I>(&self, group_id: I, swap_group_id: Option<String>) -> Result<()>
    where
        I: Into<String>,
    {
        let mut params = vec![("groupId".to_string(), group_id.into())];
        if let Some(swap) = swap_group_id {
            params.push(("swapGroupId".to_string(), swap));
        }

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/group?{}", query))
            .await?;
        Ok(())
    }

    /// Add a user to a group (async)
    ///
    /// Adds the specified user to the specified group.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group or user does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-user-post)
    /// for more information
    pub async fn add_user<G, U>(&self, group_id: G, account_id: U) -> Result<serde_json::Value>
    where
        G: Into<String>,
        U: Into<String>,
    {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("groupId", &group_id.into())
            .finish();

        let request = AddUserToGroup {
            account_id: account_id.into(),
        };

        self.jira
            .post("api", &format!("/group/user?{}", query), request)
            .await
    }

    /// Remove a user from a group (async)
    ///
    /// Removes the specified user from the specified group.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The group or user does not exist
    /// - The user lacks site administration permissions
    ///
    /// See [Jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-groups/#api-rest-api-3-group-user-delete)
    /// for more information
    pub async fn remove_user<G, U>(&self, group_id: G, account_id: U) -> Result<()>
    where
        G: Into<String>,
        U: Into<String>,
    {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("groupId", &group_id.into())
            .append_pair("accountId", &account_id.into())
            .finish();

        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/group/user?{}", query))
            .await?;
        Ok(())
    }
}
