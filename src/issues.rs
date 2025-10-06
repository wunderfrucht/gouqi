//! Interfaces for accessing and managing issues

// Third party
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::form_urlencoded;

// Ours
use crate::attachments::AttachmentResponse;
use crate::relationships::{GraphOptions, IssueRelationships, RelationshipGraph};
use crate::sync::Jira;
use crate::{
    Board, Changelog, Comment, Issue, IssueType, Priority, Project, Result, SearchOptions,
};

#[cfg(feature = "async")]
use futures::Future;
#[cfg(feature = "async")]
use futures::stream::Stream;
#[cfg(feature = "async")]
use std::pin::Pin;

/// Issue options
#[derive(Debug)]
pub struct Issues {
    jira: Jira,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Assignee {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Component {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_assignee: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_assignee_type_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<u64>,
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
}

impl Component {
    /// Create a new Component with basic fields
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Component {
            id: id.into(),
            name: name.into(),
            description: None,
            lead: None,
            assignee_type: None,
            assignee: None,
            real_assignee_type: None,
            real_assignee: None,
            is_assignee_type_valid: None,
            project: None,
            project_id: None,
            self_link: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub assignee: Assignee,
    pub components: Vec<Component>,
    pub description: String,
    pub environment: String,
    pub issuetype: IssueType,
    pub priority: Priority,
    pub project: Project,
    pub reporter: Assignee,
    pub summary: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateIssue {
    pub fields: Fields,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateCustomIssue<CustomFields> {
    pub fields: CustomFields,
}

#[derive(Debug, Deserialize)]
pub struct CreateResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditIssue<T: Serialize> {
    pub fields: BTreeMap<String, T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditCustomIssue<CustomFields> {
    pub fields: CustomFields,
}

/// Options for updating issues
///
/// These options control various aspects of the issue update operation,
/// including notifications, security overrides, and response behavior.
///
/// # Examples
///
/// ```rust
/// use gouqi::issues::IssueUpdateOptions;
///
/// // Disable notifications during update
/// let options = IssueUpdateOptions::builder()
///     .notify_users(false)
///     .build();
///
/// // Return the updated issue in response
/// let options = IssueUpdateOptions::builder()
///     .return_issue(true)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct IssueUpdateOptions {
    /// Whether to send email notifications to watchers (default: true)
    pub notify_users: Option<bool>,

    /// Whether to update fields that are not on the screen (default: false)
    pub override_screen_security: Option<bool>,

    /// Whether to update fields even if they're marked as non-editable (default: false)
    pub override_editable_flag: Option<bool>,

    /// Whether to return the updated issue in the response (default: false)
    /// When true, the response will include the updated issue data
    #[allow(clippy::struct_field_names)]
    pub return_issue: bool,

    /// Fields to expand in the returned issue (only used when return_issue is true)
    pub expand: Option<Vec<String>>,
}

impl IssueUpdateOptions {
    /// Create a new builder for IssueUpdateOptions
    pub fn builder() -> IssueUpdateOptionsBuilder {
        IssueUpdateOptionsBuilder::default()
    }

    /// Convert options to query string parameters
    fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(notify) = self.notify_users {
            params.push(format!("notifyUsers={}", notify));
        }

        if let Some(override_screen) = self.override_screen_security {
            params.push(format!("overrideScreenSecurity={}", override_screen));
        }

        if let Some(override_editable) = self.override_editable_flag {
            params.push(format!("overrideEditableFlag={}", override_editable));
        }

        if self.return_issue {
            params.push("returnIssue=true".to_string());
        }

        if let Some(ref expand) = self.expand {
            if !expand.is_empty() {
                params.push(format!("expand={}", expand.join(",")));
            }
        }

        params.join("&")
    }
}

/// Builder for IssueUpdateOptions
#[derive(Debug, Default)]
pub struct IssueUpdateOptionsBuilder {
    notify_users: Option<bool>,
    override_screen_security: Option<bool>,
    override_editable_flag: Option<bool>,
    return_issue: bool,
    expand: Option<Vec<String>>,
}

impl IssueUpdateOptionsBuilder {
    /// Set whether to send notifications to watchers
    ///
    /// When set to false, users watching the issue will not receive email notifications.
    /// This is useful for bulk operations or automated updates.
    pub fn notify_users(mut self, notify: bool) -> Self {
        self.notify_users = Some(notify);
        self
    }

    /// Set whether to override screen security
    ///
    /// When set to true, allows updating fields that are not visible on the edit screen.
    pub fn override_screen_security(mut self, override_security: bool) -> Self {
        self.override_screen_security = Some(override_security);
        self
    }

    /// Set whether to override the editable flag
    ///
    /// When set to true, allows updating fields even if they're marked as non-editable.
    pub fn override_editable_flag(mut self, override_editable: bool) -> Self {
        self.override_editable_flag = Some(override_editable);
        self
    }

    /// Set whether to return the updated issue in the response
    ///
    /// When set to true, the API will return the updated issue data.
    /// This is required for the `update_and_return` method.
    pub fn return_issue(mut self, return_issue: bool) -> Self {
        self.return_issue = return_issue;
        self
    }

    /// Set which fields to expand in the returned issue
    ///
    /// Only used when return_issue is true.
    /// Common values: "renderedFields", "names", "schema", "transitions", "operations", "changelog"
    pub fn expand(mut self, expand: Vec<String>) -> Self {
        self.expand = Some(expand);
        self
    }

    /// Build the IssueUpdateOptions
    pub fn build(self) -> IssueUpdateOptions {
        IssueUpdateOptions {
            notify_users: self.notify_users,
            override_screen_security: self.override_screen_security,
            override_editable_flag: self.override_editable_flag,
            return_issue: self.return_issue,
            expand: self.expand,
        }
    }
}

/// Options for retrieving issues
///
/// This struct allows you to customize how issues are fetched from Jira.
/// You can control which fields are returned, what data to expand, and other options.
///
/// # Examples
///
/// ```
/// use gouqi::issues::IssueGetOptions;
///
/// let options = IssueGetOptions::builder()
///     .fields(vec!["summary".to_string(), "status".to_string()])
///     .expand(vec!["changelog".to_string()])
///     .build();
/// ```
#[derive(Default, Debug, Clone)]
pub struct IssueGetOptions {
    /// Fields to include in the response
    pub fields: Option<Vec<String>>,

    /// Fields to expand in the response
    pub expand: Option<Vec<String>>,

    /// Properties to include in the response
    pub properties: Option<Vec<String>>,

    /// Whether to include update history
    pub update_history: Option<bool>,

    /// Whether to use field keys instead of field IDs
    pub fields_by_keys: Option<bool>,
}

impl IssueGetOptions {
    /// Create a new builder for IssueGetOptions
    pub fn builder() -> IssueGetOptionsBuilder {
        IssueGetOptionsBuilder::default()
    }

    /// Convert options to query string parameters
    pub fn to_query_string(&self) -> Option<String> {
        use url::form_urlencoded;
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        let mut has_params = false;

        if let Some(ref fields) = self.fields {
            serializer.append_pair("fields", &fields.join(","));
            has_params = true;
        }
        if let Some(ref expand) = self.expand {
            serializer.append_pair("expand", &expand.join(","));
            has_params = true;
        }
        if let Some(ref properties) = self.properties {
            serializer.append_pair("properties", &properties.join(","));
            has_params = true;
        }
        if let Some(update_history) = self.update_history {
            serializer.append_pair("updateHistory", &update_history.to_string());
            has_params = true;
        }
        if let Some(fields_by_keys) = self.fields_by_keys {
            serializer.append_pair("fieldsByKeys", &fields_by_keys.to_string());
            has_params = true;
        }

        if has_params {
            Some(serializer.finish())
        } else {
            None
        }
    }
}

/// Builder for IssueGetOptions
#[derive(Default, Debug)]
pub struct IssueGetOptionsBuilder {
    fields: Option<Vec<String>>,
    expand: Option<Vec<String>>,
    properties: Option<Vec<String>>,
    update_history: Option<bool>,
    fields_by_keys: Option<bool>,
}

impl IssueGetOptionsBuilder {
    /// Set which fields to include in the response
    ///
    /// By default, all navigable fields are returned.
    /// You can specify a subset of fields to reduce the response size.
    pub fn fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fields = Some(fields.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set which fields to expand in the response
    ///
    /// Common values: "renderedFields", "names", "schema", "transitions", "operations", "changelog"
    pub fn expand<I, S>(mut self, expand: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.expand = Some(expand.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set which properties to include in the response
    pub fn properties<I, S>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.properties = Some(properties.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set whether to include update history
    pub fn update_history(mut self, value: bool) -> Self {
        self.update_history = Some(value);
        self
    }

    /// Set whether to use field keys instead of field IDs
    pub fn fields_by_keys(mut self, value: bool) -> Self {
        self.fields_by_keys = Some(value);
        self
    }

    /// Build the IssueGetOptions
    pub fn build(self) -> IssueGetOptions {
        IssueGetOptions {
            fields: self.fields,
            expand: self.expand,
            properties: self.properties,
            update_history: self.update_history,
            fields_by_keys: self.fields_by_keys,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct IssueResults {
    pub expand: Option<String>,
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    pub total: u64,
    pub issues: Vec<Issue>,
}

/// Options for deleting issues
///
/// These options control the behavior when deleting an issue.
///
/// # Examples
///
/// ```rust
/// use gouqi::issues::IssueDeleteOptions;
///
/// // Delete an issue and its subtasks
/// let options = IssueDeleteOptions::builder()
///     .delete_subtasks(true)
///     .build();
/// ```
#[derive(Default, Debug, Clone)]
pub struct IssueDeleteOptions {
    /// Whether to delete subtasks when deleting parent issue
    pub delete_subtasks: Option<bool>,
}

impl IssueDeleteOptions {
    /// Create a new builder for IssueDeleteOptions
    pub fn builder() -> IssueDeleteOptionsBuilder {
        IssueDeleteOptionsBuilder::default()
    }

    /// Convert options to query string parameters
    fn to_query_string(&self) -> String {
        let mut params = vec![];
        if let Some(delete_subtasks) = self.delete_subtasks {
            params.push(format!("deleteSubtasks={}", delete_subtasks));
        }
        params.join("&")
    }
}

/// Builder for IssueDeleteOptions
#[derive(Default, Debug)]
pub struct IssueDeleteOptionsBuilder {
    delete_subtasks: Option<bool>,
}

impl IssueDeleteOptionsBuilder {
    /// Set whether to delete subtasks when deleting parent issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::issues::IssueDeleteOptions;
    /// let options = IssueDeleteOptions::builder()
    ///     .delete_subtasks(true)
    ///     .build();
    /// ```
    pub fn delete_subtasks(mut self, value: bool) -> Self {
        self.delete_subtasks = Some(value);
        self
    }

    /// Build the IssueDeleteOptions
    pub fn build(self) -> IssueDeleteOptions {
        IssueDeleteOptions {
            delete_subtasks: self.delete_subtasks,
        }
    }
}

/// Options for how to adjust the remaining estimate when logging work
///
/// When logging work on an issue, you can control how the remaining estimate is adjusted.
/// See [Jira API documentation](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-addWorklog)
/// for more information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustEstimate {
    /// Automatically adjust the remaining estimate (default behavior)
    /// The remaining estimate will be reduced by the time logged
    Auto,

    /// Set a new remaining estimate value
    /// Provide the new estimate value (e.g., "2h", "1d 4h")
    New(String),

    /// Reduce the remaining estimate by the specified amount
    /// Provide the amount to reduce by (e.g., "30m", "1h")
    Manual(String),

    /// Do not adjust the remaining estimate at all
    Leave,
}

impl Default for AdjustEstimate {
    fn default() -> Self {
        Self::Auto
    }
}

/// Options for worklog operations
///
/// Controls various parameters when adding or updating worklogs, such as how to adjust
/// the remaining time estimate and whether to notify users.
///
/// # Examples
///
/// ```rust
/// use gouqi::issues::{WorklogOptions, AdjustEstimate};
///
/// // Set a new estimate when logging work
/// let options = WorklogOptions::builder()
///     .adjust_estimate(AdjustEstimate::New("2h".to_string()))
///     .notify_users(false)
///     .build();
///
/// // Reduce estimate by a specific amount
/// let options = WorklogOptions::builder()
///     .adjust_estimate(AdjustEstimate::Manual("30m".to_string()))
///     .build();
///
/// // Don't adjust the estimate at all
/// let options = WorklogOptions::builder()
///     .adjust_estimate(AdjustEstimate::Leave)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct WorklogOptions {
    /// How to adjust the remaining time estimate
    pub adjust_estimate: Option<AdjustEstimate>,

    /// Whether to send email notifications to watchers (default: true)
    pub notify_users: Option<bool>,

    /// Whether to update fields even if they're marked as non-editable (default: false)
    pub override_editable_flag: Option<bool>,
}

impl WorklogOptions {
    /// Create a new builder for WorklogOptions
    pub fn builder() -> WorklogOptionsBuilder {
        WorklogOptionsBuilder::default()
    }

    /// Convert options to query string parameters
    fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(ref adjust) = self.adjust_estimate {
            match adjust {
                AdjustEstimate::Auto => {
                    params.push("adjustEstimate=auto".to_string());
                }
                AdjustEstimate::New(value) => {
                    params.push("adjustEstimate=new".to_string());
                    params.push(format!(
                        "newEstimate={}",
                        form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()
                    ));
                }
                AdjustEstimate::Manual(value) => {
                    params.push("adjustEstimate=manual".to_string());
                    params.push(format!(
                        "reduceBy={}",
                        form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()
                    ));
                }
                AdjustEstimate::Leave => {
                    params.push("adjustEstimate=leave".to_string());
                }
            }
        }

        if let Some(notify) = self.notify_users {
            params.push(format!("notifyUsers={}", notify));
        }

        if let Some(override_editable) = self.override_editable_flag {
            params.push(format!("overrideEditableFlag={}", override_editable));
        }

        params.join("&")
    }
}

/// Builder for WorklogOptions
#[derive(Debug, Default)]
pub struct WorklogOptionsBuilder {
    adjust_estimate: Option<AdjustEstimate>,
    notify_users: Option<bool>,
    override_editable_flag: Option<bool>,
}

impl WorklogOptionsBuilder {
    /// Set how to adjust the remaining estimate
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::issues::{WorklogOptions, AdjustEstimate};
    /// // Set a new estimate
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::New("2h".to_string()))
    ///     .build();
    ///
    /// // Reduce by a specific amount
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::Manual("30m".to_string()))
    ///     .build();
    ///
    /// // Don't adjust at all
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::Leave)
    ///     .build();
    /// ```
    pub fn adjust_estimate(mut self, adjust: AdjustEstimate) -> Self {
        self.adjust_estimate = Some(adjust);
        self
    }

    /// Set whether to send notifications to watchers
    ///
    /// When set to false, users watching the issue will not receive email notifications.
    /// This is useful for bulk operations or automated updates.
    pub fn notify_users(mut self, notify: bool) -> Self {
        self.notify_users = Some(notify);
        self
    }

    /// Set whether to override the editable flag
    ///
    /// When set to true, allows updating fields even if they're marked as non-editable.
    pub fn override_editable_flag(mut self, override_editable: bool) -> Self {
        self.override_editable_flag = Some(override_editable);
        self
    }

    /// Build the WorklogOptions
    pub fn build(self) -> WorklogOptions {
        WorklogOptions {
            adjust_estimate: self.adjust_estimate,
            notify_users: self.notify_users,
            override_editable_flag: self.override_editable_flag,
        }
    }
}

/// Request body for adding a comment (V2 API - plain text)
#[derive(Debug, Serialize)]
pub struct AddComment {
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<crate::rep::Visibility>,
}

impl AddComment {
    /// Create a new comment with plain text
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            visibility: None,
        }
    }

    /// Set visibility restrictions for the comment
    pub fn with_visibility(mut self, visibility: crate::rep::Visibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

/// Request body for adding a comment (V3 API - ADF format)
#[derive(Debug, Serialize)]
pub struct AddCommentAdf {
    pub body: crate::rep::AdfDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<crate::rep::Visibility>,
}

impl AddCommentAdf {
    /// Create a new comment from plain text (converts to ADF)
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            body: crate::rep::AdfDocument::from_text(text),
            visibility: None,
        }
    }

    /// Create a comment from an ADF document
    pub fn from_adf(body: crate::rep::AdfDocument) -> Self {
        Self {
            body,
            visibility: None,
        }
    }

    /// Set visibility restrictions for the comment
    pub fn with_visibility(mut self, visibility: crate::rep::Visibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

#[derive(Debug, Serialize)]
pub struct AssignRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Watchers {
    #[serde(rename = "self")]
    pub self_link: String,
    pub watchers: Vec<crate::User>,
    #[serde(rename = "watchCount")]
    pub watch_count: u32,
    #[serde(rename = "isWatching")]
    pub is_watching: bool,
}

#[derive(Serialize, Debug)]
pub struct BulkCreateRequest {
    #[serde(rename = "issueUpdates")]
    pub issue_updates: Vec<CreateIssue>,
}

#[derive(Serialize, Debug)]
pub struct BulkUpdateRequest {
    #[serde(rename = "issueUpdates")]
    pub issue_updates: Vec<BulkIssueUpdate>,
}

#[derive(Serialize, Debug)]
pub struct BulkIssueUpdate {
    pub key: String,
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct BulkCreateResponse {
    pub issues: Vec<CreateResponse>,
    pub errors: Vec<BulkError>,
}

#[derive(Deserialize, Debug)]
pub struct BulkUpdateResponse {
    pub issues: Vec<Issue>,
    pub errors: Vec<BulkError>,
}

#[derive(Deserialize, Debug)]
pub struct BulkError {
    pub status: u16,
    #[serde(rename = "elementErrors")]
    pub element_errors: crate::Errors,
    #[serde(rename = "failedElementNumber")]
    pub failed_element_number: Option<u32>,
}

impl Issues {
    pub fn new(jira: &Jira) -> Issues {
        Issues { jira: jira.clone() }
    }

    /// Get a single issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/issue)
    /// for more information
    pub fn get<I>(&self, id: I) -> Result<Issue>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/issue/{}", id.into()))
    }

    /// Get a single issue with custom options
    ///
    /// This method allows you to specify which fields to retrieve, what to expand, and other options.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    /// use gouqi::issues::IssueGetOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let options = IssueGetOptions::builder()
    ///     .fields(vec!["summary".to_string(), "status".to_string()])
    ///     .expand(vec!["changelog".to_string()])
    ///     .build();
    /// let issue = jira.issues().get_with_options("ISSUE-123", &options).unwrap();
    /// ```
    pub fn get_with_options<I>(&self, id: I, options: &IssueGetOptions) -> Result<Issue>
    where
        I: Into<String>,
    {
        let url = if let Some(query) = options.to_query_string() {
            format!("/issue/{}?{}", id.into(), query)
        } else {
            format!("/issue/{}", id.into())
        };
        self.jira.get("api", &url)
    }

    /// Get a single custom issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/issue)
    /// for more information
    pub fn get_custom_issue<I, D>(&self, id: I) -> Result<EditCustomIssue<D>>
    where
        D: serde::de::DeserializeOwned,
        I: Into<String>,
    {
        self.jira.get("api", &format!("/issue/{}", id.into()))
    }

    /// Create a new issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-createIssue)
    /// for more information
    pub fn create(&self, data: CreateIssue) -> Result<CreateResponse> {
        self.jira.post("api", "/issue", data)
    }

    /// Create a new custom issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-createIssue)
    /// for more information
    pub fn create_from_custom_issue<T: serde::Serialize>(
        &self,
        data: CreateCustomIssue<T>,
    ) -> Result<CreateResponse> {
        self.jira.post("api", "/issue", data)
    }

    /// Update an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn update<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira.put("api", &format!("/issue/{}", id.into()), data)
    }

    /// Update an issue with options
    ///
    /// This method allows fine-grained control over the update operation, including
    /// disabling notifications, overriding security settings, and controlling response behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # use gouqi::issues::{EditIssue, IssueUpdateOptions};
    /// # use std::collections::BTreeMap;
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Update without sending notifications
    /// let mut fields = BTreeMap::new();
    /// fields.insert("summary".to_string(), serde_json::Value::String("New summary".to_string()));
    /// let edit = EditIssue { fields };
    ///
    /// let options = IssueUpdateOptions::builder()
    ///     .notify_users(false)
    ///     .build();
    ///
    /// jira.issues().update_with_options("PROJ-123", edit, &options)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn update_with_options<I, T>(
        &self,
        id: I,
        data: EditIssue<T>,
        options: &IssueUpdateOptions,
    ) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}", id.into())
        } else {
            format!("/issue/{}?{}", id.into(), query_string)
        };
        self.jira.put("api", &path, data)
    }

    /// Update an issue and return the updated issue
    ///
    /// This method updates an issue and returns the updated issue data in the response.
    /// This is useful when you need to see the result of the update immediately.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # use gouqi::issues::{EditIssue, IssueUpdateOptions};
    /// # use std::collections::BTreeMap;
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Update and get the updated issue back
    /// let mut fields = BTreeMap::new();
    /// fields.insert("summary".to_string(), serde_json::Value::String("Updated summary".to_string()));
    /// let edit = EditIssue { fields };
    ///
    /// let options = IssueUpdateOptions::builder()
    ///     .notify_users(false)
    ///     .return_issue(true)
    ///     .expand(vec!["changelog".to_string()])
    ///     .build();
    ///
    /// let updated_issue = jira.issues().update_and_return("PROJ-123", edit, &options)?;
    /// println!("Updated: {}", updated_issue.key);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn update_and_return<I, T>(
        &self,
        id: I,
        data: EditIssue<T>,
        options: &IssueUpdateOptions,
    ) -> Result<Issue>
    where
        I: Into<String>,
        T: Serialize,
    {
        let mut opts = options.clone();
        opts.return_issue = true; // Ensure return_issue is set

        let query_string = opts.to_query_string();
        let path = format!("/issue/{}?{}", id.into(), query_string);
        self.jira.put("api", &path, data)
    }

    /// Edit an issue
    ///
    /// # Deprecated
    ///
    /// Use [`Issues::update`] instead. This method will be removed in a future version.
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    #[deprecated(
        since = "0.16.0",
        note = "Use `update` instead for consistency with REST conventions"
    )]
    pub fn edit<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.update(id, data)
    }

    /// Update a custom issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn update_custom_issue<I, T>(&self, id: I, data: EditCustomIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira.put("api", &format!("/issue/{}", id.into()), data)
    }

    /// Edit a custom issue
    ///
    /// # Deprecated
    ///
    /// Use [`Issues::update_custom_issue`] instead. This method will be removed in a future version.
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    #[deprecated(
        since = "0.16.0",
        note = "Use `update_custom_issue` instead for consistency with REST conventions"
    )]
    pub fn edit_custom_issue<I, T>(&self, id: I, data: EditCustomIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.update_custom_issue(id, data)
    }

    /// Returns a single page of issue results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard)
    /// for more information
    pub fn list(&self, board: &Board, options: &SearchOptions) -> Result<IssueResults> {
        let mut path = vec![format!("/board/{}/issue", board.id)];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<IssueResults>("agile", path.join("?").as_ref())
    }

    /// Returns a type which may be used to iterate over consecutive pages of results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard)
    /// for more information
    pub fn iter<'a>(&self, board: &'a Board, options: &'a SearchOptions) -> Result<IssuesIter<'a>> {
        IssuesIter::new(board, options, &self.jira)
    }

    /// Add a comment to an issue
    ///
    /// Automatically detects whether to use V2 (plain text) or V3 (ADF) format
    /// based on the Jira deployment type. For Jira Cloud, uses V3/ADF format.
    /// For Server/Data Center, uses V2/plain text format.
    ///
    /// See [V2 docs](https://developer.atlassian.com/server/jira/platform/jira-rest-api-example-add-comment-8946422/)
    /// and [V3 docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-comments/)
    /// for more information
    pub fn comment<K>(&self, key: K, data: AddComment) -> Result<Comment>
    where
        K: Into<String>,
    {
        use crate::core::SearchApiVersion;

        let issue_key = key.into();

        // Detect API version (same logic as search API)
        match self.jira.core.get_search_api_version() {
            SearchApiVersion::V3 => {
                // V3 API requires ADF format
                let adf_comment = if let Some(visibility) = data.visibility {
                    AddCommentAdf::from_text(data.body).with_visibility(visibility)
                } else {
                    AddCommentAdf::from_text(data.body)
                };

                self.jira.post_versioned(
                    "api",
                    Some("3"),
                    format!("/issue/{}/comment", issue_key).as_ref(),
                    adf_comment,
                )
            }
            _ => {
                // V2 API uses plain text
                self.jira.post_versioned(
                    "api",
                    Some("latest"),
                    format!("/issue/{}/comment", issue_key).as_ref(),
                    data,
                )
            }
        }
    }

    pub fn changelog<K>(&self, key: K) -> Result<Changelog>
    where
        K: Into<String>,
    {
        self.jira
            .get("api", format!("/issue/{}/changelog", key.into()).as_ref())
    }

    /// Extract relationship graph from Jira to specified depth
    ///
    /// This method traverses issue relationships breadth-first starting from
    /// the root issue and builds a declarative relationship graph that can be
    /// used for analysis or applied to other Jira instances.
    ///
    /// # Arguments
    ///
    /// * `root_issue` - The issue key to start traversal from
    /// * `depth` - Maximum depth to traverse (0 = root issue only)
    /// * `options` - Optional configuration for graph extraction
    ///
    /// # Returns
    ///
    /// A `RelationshipGraph` containing all discovered relationships
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Get all relationships 2 levels deep from PROJ-123
    /// let graph = jira.issues()
    ///     .get_relationship_graph("PROJ-123", 2, None)?;
    ///
    /// // Get only blocking relationships
    /// use gouqi::relationships::GraphOptions;
    /// let options = GraphOptions {
    ///     include_types: Some(vec!["blocks".to_string(), "blocked_by".to_string()]),
    ///     ..Default::default()
    /// };
    /// let blocking_graph = jira.issues()
    ///     .get_relationship_graph("PROJ-123", 1, Some(options))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_relationship_graph(
        &self,
        root_issue: &str,
        depth: u32,
        options: Option<GraphOptions>,
    ) -> Result<RelationshipGraph> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let options = options.unwrap_or_default();
        let mut graph = RelationshipGraph::new("jira".to_string());
        graph.metadata.root_issue = Some(root_issue.to_string());
        graph.metadata.max_depth = depth;

        // BFS traversal
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut depth_map = HashMap::new();

        queue.push_back(root_issue.to_string());
        depth_map.insert(root_issue.to_string(), 0);

        while let Some(current_issue) = queue.pop_front() {
            let current_depth = depth_map[&current_issue];

            if visited.contains(&current_issue) {
                continue;
            }
            visited.insert(current_issue.clone());

            // Get the issue details
            let issue = match self.get(&current_issue) {
                Ok(issue) => issue,
                Err(_) => {
                    // Issue not found or not accessible, skip
                    continue;
                }
            };

            // Extract relationships from the issue
            let relationships = self.extract_relationships_from_issue(&issue, &options)?;

            // Add to graph
            graph.add_issue(current_issue.clone(), relationships.clone());

            // If we haven't reached max depth, add related issues to queue
            if current_depth < depth {
                let related_issues = relationships.get_all_related();
                for related_issue in related_issues {
                    if !depth_map.contains_key(&related_issue) {
                        depth_map.insert(related_issue.clone(), current_depth + 1);
                        queue.push_back(related_issue);
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Extract relationships from a single issue
    fn extract_relationships_from_issue(
        &self,
        issue: &Issue,
        options: &GraphOptions,
    ) -> Result<IssueRelationships> {
        let mut relationships = IssueRelationships::new();

        // Extract issue links
        if let Some(Ok(links)) = issue.links() {
            for link in links {
                let link_type_name = &link.link_type.name;

                // Check if this link type should be included
                if let Some(ref include_types) = options.include_types {
                    if !include_types.contains(link_type_name) {
                        continue;
                    }
                }
                if let Some(ref exclude_types) = options.exclude_types {
                    if exclude_types.contains(link_type_name) {
                        continue;
                    }
                }

                // Map Jira link types to our standard types
                let (outward_type, inward_type) = self.map_link_type(link_type_name);

                // Add outward relationship
                if let Some(ref outward_issue) = link.outward_issue {
                    if options.bidirectional || Some(&issue.key) != Some(&outward_issue.key) {
                        relationships.add_relationship(&outward_type, outward_issue.key.clone());
                    }
                }

                // Add inward relationship
                if let Some(ref inward_issue) = link.inward_issue {
                    if options.bidirectional || Some(&issue.key) != Some(&inward_issue.key) {
                        relationships.add_relationship(&inward_type, inward_issue.key.clone());
                    }
                }

                // Add to custom if not a standard type
                if !self.is_standard_link_type(link_type_name) && options.include_custom {
                    if let Some(ref outward_issue) = link.outward_issue {
                        relationships.add_relationship(
                            &format!("custom_{}", link_type_name.to_lowercase()),
                            outward_issue.key.clone(),
                        );
                    }
                    if let Some(ref inward_issue) = link.inward_issue {
                        relationships.add_relationship(
                            &format!("custom_{}_inward", link_type_name.to_lowercase()),
                            inward_issue.key.clone(),
                        );
                    }
                }
            }
        }

        // Extract parent-child relationships
        if let Some(parent_issue) = issue.parent() {
            relationships.parent = Some(parent_issue.key);
        }

        // Extract epic relationships (if available in custom fields)
        // This would need to be customized based on the Jira instance configuration
        if let Some(epic_link) = self.extract_epic_link(issue) {
            relationships.epic = Some(epic_link);
        }

        Ok(relationships)
    }

    /// Map Jira link type names to our standard relationship types
    fn map_link_type(&self, link_type_name: &str) -> (String, String) {
        match link_type_name.to_lowercase().as_str() {
            "blocks" => ("blocks".to_string(), "blocked_by".to_string()),
            "duplicate" | "duplicates" => ("duplicates".to_string(), "duplicated_by".to_string()),
            "relates" | "relates to" => ("relates_to".to_string(), "relates_to".to_string()),
            "clones" => ("duplicates".to_string(), "duplicated_by".to_string()),
            "causes" => ("blocks".to_string(), "blocked_by".to_string()),
            _ => (
                format!("custom_{}", link_type_name.to_lowercase()),
                format!("custom_{}_inward", link_type_name.to_lowercase()),
            ),
        }
    }

    /// Check if a link type is one of our standard types
    fn is_standard_link_type(&self, link_type_name: &str) -> bool {
        matches!(
            link_type_name.to_lowercase().as_str(),
            "blocks" | "duplicate" | "duplicates" | "relates" | "relates to" | "clones" | "causes"
        )
    }

    /// Extract epic link from issue (customize based on your Jira configuration)
    fn extract_epic_link(&self, issue: &Issue) -> Option<String> {
        // This is a common custom field for Epic Link
        // You may need to adjust the field name based on your Jira configuration
        issue
            .field::<String>("customfield_10014")
            .and_then(|result| result.ok())
            .or_else(|| {
                issue
                    .field::<String>("customfield_10008")
                    .and_then(|result| result.ok())
            })
            .or_else(|| {
                issue
                    .field::<String>("Epic Link")
                    .and_then(|result| result.ok())
            })
    }

    /// Get current relationships for multiple issues efficiently
    ///
    /// This is more efficient than calling `get_relationship_graph` for each issue
    /// individually when you need relationships for a known set of issues.
    ///
    /// # Arguments
    ///
    /// * `issue_keys` - List of issue keys to get relationships for
    /// * `options` - Optional configuration for relationship extraction
    ///
    /// # Returns
    ///
    /// A `RelationshipGraph` containing relationships for all specified issues
    pub fn get_bulk_relationships(
        &self,
        issue_keys: &[String],
        options: Option<GraphOptions>,
    ) -> Result<RelationshipGraph> {
        let options = options.unwrap_or_default();
        let mut graph = RelationshipGraph::new("jira_bulk".to_string());
        graph.metadata.max_depth = 0; // Direct relationships only

        for issue_key in issue_keys {
            match self.get(issue_key) {
                Ok(issue) => {
                    let relationships = self.extract_relationships_from_issue(&issue, &options)?;
                    graph.add_issue(issue_key.clone(), relationships);
                }
                Err(_) => {
                    // Issue not found or not accessible, skip but could log
                    continue;
                }
            }
        }

        Ok(graph)
    }

    /// Delete an issue
    ///
    /// Deletes an issue from Jira. The issue must exist and the user must have
    /// permission to delete it.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Delete an issue
    /// jira.issues().delete("PROJ-123")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The issue does not exist
    /// - The user lacks permission to delete the issue
    /// - The issue cannot be deleted due to workflow restrictions
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-deleteIssue)
    /// for more information
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/issue/{}", id.into()))?;
        Ok(())
    }

    /// Delete an issue with options
    ///
    /// Deletes an issue from Jira with additional options such as deleting subtasks.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    /// * `options` - Options for the delete operation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gouqi::{Jira, Credentials};
    /// # use gouqi::issues::IssueDeleteOptions;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let jira = Jira::new("https://jira.example.com", Credentials::Basic("user".to_string(), "token".to_string()))?;
    /// let options = IssueDeleteOptions::builder()
    ///     .delete_subtasks(true)
    ///     .build();
    /// jira.issues().delete_with_options("PROJ-123", options)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The issue does not exist
    /// - The user lacks permission to delete the issue
    /// - The issue cannot be deleted due to workflow restrictions
    ///
    /// # Panics
    ///
    /// This function will panic if the issue cannot be deleted due to workflow restrictions
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-deleteIssue)
    /// for more information
    pub fn delete_with_options<I>(&self, id: I, options: IssueDeleteOptions) -> Result<()>
    where
        I: Into<String>,
    {
        let query = options.to_query_string();
        let path = if query.is_empty() {
            format!("/issue/{}", id.into())
        } else {
            format!("/issue/{}?{}", id.into(), query)
        };
        self.jira.delete::<crate::EmptyResponse>("api", &path)?;
        Ok(())
    }

    /// Archive an issue
    ///
    /// Archives an issue in Jira. Archived issues are hidden from most views
    /// but can be restored later if needed.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Archive an issue
    /// jira.issues().archive("PROJ-123")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The issue does not exist
    /// - The user lacks permission to archive the issue
    /// - The issue is already archived
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-archiveIssue)
    /// for more information
    pub fn archive<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/archive", id.into()), ())
    }

    /// Get all worklogs for an issue
    ///
    /// Returns a paginated list of all work logs for the specified issue.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let worklogs = jira.issues().get_worklogs("PROJ-123")?;
    /// for worklog in worklogs.worklogs {
    ///     println!("Worklog: {} - {}", worklog.id, worklog.time_spent.unwrap_or_default());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_worklogs<K>(&self, issue_key: K) -> Result<crate::WorklogList>
    where
        K: Into<String>,
    {
        self.jira
            .get("api", &format!("/issue/{}/worklog", issue_key.into()))
    }

    /// Get a specific worklog by ID
    ///
    /// Returns details of a specific worklog entry for an issue.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog_id` - The ID of the worklog
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let worklog = jira.issues().get_worklog("PROJ-123", "10001")?;
    /// println!("Time spent: {}", worklog.time_spent.unwrap_or_default());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_worklog<K, W>(&self, issue_key: K, worklog_id: W) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira.get(
            "api",
            &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
        )
    }

    /// Add a worklog to an issue
    ///
    /// Creates a new worklog entry for the specified issue.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog` - The worklog data to create
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira, WorklogInput};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Log 2 hours (7200 seconds)
    /// let worklog = WorklogInput::new(7200)
    ///     .with_comment("Fixed the bug");
    ///
    /// let created = jira.issues().add_worklog("PROJ-123", worklog)?;
    /// println!("Created worklog: {}", created.id);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_worklog<K>(
        &self,
        issue_key: K,
        worklog: crate::WorklogInput,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
    {
        self.jira.post(
            "api",
            &format!("/issue/{}/worklog", issue_key.into()),
            worklog,
        )
    }

    /// Update an existing worklog
    ///
    /// Updates a worklog entry for the specified issue.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog_id` - The ID of the worklog to update
    /// * `worklog` - The updated worklog data
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira, WorklogInput};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let worklog = WorklogInput::new(3600)  // 1 hour
    ///     .with_comment("Updated time estimate");
    ///
    /// let updated = jira.issues().update_worklog("PROJ-123", "10001", worklog)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn update_worklog<K, W>(
        &self,
        issue_key: K,
        worklog_id: W,
        worklog: crate::WorklogInput,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira.put(
            "api",
            &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
            worklog,
        )
    }

    /// Add a worklog with options for time tracking and notifications
    ///
    /// This method allows you to control how the remaining estimate is adjusted when logging work,
    /// and whether to send notifications to watchers.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog` - The worklog data to add
    /// * `options` - Options controlling estimate adjustment and notifications
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira, WorklogInput};
    /// # use gouqi::issues::{WorklogOptions, AdjustEstimate};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Log work and set a new remaining estimate
    /// let worklog = WorklogInput::new(7200).with_comment("Implemented feature");
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::New("1d".to_string()))
    ///     .notify_users(false)
    ///     .build();
    ///
    /// let created = jira.issues().add_worklog_with_options("PROJ-123", worklog, &options)?;
    /// println!("Created worklog: {}", created.id);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_worklog_with_options<K>(
        &self,
        issue_key: K,
        worklog: crate::WorklogInput,
        options: &WorklogOptions,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}/worklog", issue_key.into())
        } else {
            format!("/issue/{}/worklog?{}", issue_key.into(), query_string)
        };
        self.jira.post("api", &path, worklog)
    }

    /// Update a worklog with options for time tracking and notifications
    ///
    /// This method allows you to control how the remaining estimate is adjusted when updating work,
    /// and whether to send notifications to watchers.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog_id` - The ID of the worklog to update
    /// * `worklog` - The updated worklog data
    /// * `options` - Options controlling estimate adjustment and notifications
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira, WorklogInput};
    /// # use gouqi::issues::{WorklogOptions, AdjustEstimate};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Update work and reduce the estimate by a specific amount
    /// let worklog = WorklogInput::new(3600).with_comment("Updated time");
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::Manual("30m".to_string()))
    ///     .notify_users(false)
    ///     .build();
    ///
    /// let updated = jira.issues().update_worklog_with_options("PROJ-123", "10001", worklog, &options)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn update_worklog_with_options<K, W>(
        &self,
        issue_key: K,
        worklog_id: W,
        worklog: crate::WorklogInput,
        options: &WorklogOptions,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into())
        } else {
            format!(
                "/issue/{}/worklog/{}?{}",
                issue_key.into(),
                worklog_id.into(),
                query_string
            )
        };
        self.jira.put("api", &path, worklog)
    }

    /// Delete a worklog
    ///
    /// Deletes a worklog entry from an issue.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog_id` - The ID of the worklog to delete
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// jira.issues().delete_worklog("PROJ-123", "10001")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete_worklog<K, W>(&self, issue_key: K, worklog_id: W) -> Result<()>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira.delete::<crate::EmptyResponse>(
            "api",
            &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
        )?;
        Ok(())
    }

    /// Assign or unassign an issue
    ///
    /// Assigns an issue to a specific user or unassigns it by passing `None`.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    /// * `assignee` - The username to assign to, or `None` to unassign
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Assign an issue to a user
    /// jira.issues().assign("PROJ-123", Some("johndoe".to_string()))?;
    ///
    /// // Unassign an issue
    /// jira.issues().assign("PROJ-123", None)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The issue does not exist
    /// - The user lacks permission to assign the issue
    /// - The specified assignee is invalid or doesn't exist
    /// - The assignee cannot be assigned to issues in this project
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-assign)
    /// for more information
    pub fn assign<I>(&self, id: I, assignee: Option<String>) -> Result<()>
    where
        I: Into<String>,
    {
        let assign_request = AssignRequest { assignee };
        self.jira.put(
            "api",
            &format!("/issue/{}/assignee", id.into()),
            assign_request,
        )
    }

    /// Get list of users watching an issue
    ///
    /// Returns information about all users watching the specified issue,
    /// including the total watch count and whether the current user is watching.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let watchers = jira.issues().get_watchers("PROJ-123")?;
    /// println!("Total watchers: {}", watchers.watch_count);
    /// println!("I'm watching: {}", watchers.is_watching);
    ///
    /// for watcher in &watchers.watchers {
    ///     println!("Watcher: {}", watcher.display_name);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-getIssueWatchers)
    /// for more information
    pub fn get_watchers<I>(&self, id: I) -> Result<Watchers>
    where
        I: Into<String>,
    {
        self.jira
            .get("api", &format!("/issue/{}/watchers", id.into()))
    }

    /// Add a watcher to an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-addWatcher)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the user cannot be added as a watcher
    pub fn add_watcher<I>(&self, id: I, username: String) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/watchers", id.into()), username)
    }

    /// Remove a watcher from an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-removeWatcher)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the watcher cannot be removed
    pub fn remove_watcher<I>(&self, id: I, username: String) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira.delete(
            "api",
            &format!("/issue/{}/watchers?username={}", id.into(), username),
        )
    }

    /// Vote for an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-addVote)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if voting fails
    pub fn vote<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/votes", id.into()), ())
    }

    /// Remove vote from an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-removeVote)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if vote removal fails
    pub fn unvote<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete("api", &format!("/issue/{}/votes", id.into()))
    }

    /// Create multiple issues in a single request
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-createIssues)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if any issue creation fails validation
    pub fn bulk_create(&self, issues: Vec<CreateIssue>) -> Result<BulkCreateResponse> {
        let bulk_request = BulkCreateRequest {
            issue_updates: issues,
        };
        self.jira.post("api", "/issue/bulk", bulk_request)
    }

    /// Update multiple issues in a single request
    ///
    /// Performs bulk updates on multiple issues efficiently in a single API call.
    /// Each issue can have different fields updated.
    ///
    /// # Arguments
    ///
    /// * `updates` - A `BulkUpdateRequest` containing all the issues to update
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # use gouqi::issues::{BulkUpdateRequest, BulkIssueUpdate};
    /// # use std::collections::BTreeMap;
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// // Update multiple issues
    /// let mut fields1 = BTreeMap::new();
    /// fields1.insert("summary".to_string(),
    ///               serde_json::Value::String("New summary".to_string()));
    ///
    /// let mut fields2 = BTreeMap::new();
    /// fields2.insert("priority".to_string(),
    ///               serde_json::json!({ "name": "High" }));
    ///
    /// let request = BulkUpdateRequest {
    ///     issue_updates: vec![
    ///         BulkIssueUpdate {
    ///             key: "PROJ-123".to_string(),
    ///             fields: fields1,
    ///         },
    ///         BulkIssueUpdate {
    ///             key: "PROJ-124".to_string(),
    ///             fields: fields2,
    ///         },
    ///     ],
    /// };
    ///
    /// let response = jira.issues().bulk_update(request)?;
    /// println!("Updated {} issues", response.issues.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Any of the issues don't exist
    /// - The user lacks permission to update any of the issues
    /// - Invalid field values are provided
    /// - Request size exceeds Jira's limits
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-updateBulkIssues)
    /// for more information
    pub fn bulk_update(&self, updates: BulkUpdateRequest) -> Result<BulkUpdateResponse> {
        self.jira.put("api", "/issue/bulk", updates)
    }

    /// Upload one or more attachments to an issue
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123")
    /// * `files` - Vector of tuples containing (filename, file_content)
    ///
    /// # Returns
    ///
    /// `Result<Vec<AttachmentResponse>>` - Array of uploaded attachment metadata
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gouqi::{Jira, Credentials};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous).unwrap();
    /// let file_content = std::fs::read("document.pdf").unwrap();
    /// let attachments = jira.issues()
    ///     .upload_attachment("PROJ-123", vec![("document.pdf", file_content)])
    ///     .unwrap();
    /// ```
    pub fn upload_attachment<I>(
        &self,
        issue_key: I,
        files: Vec<(&str, Vec<u8>)>,
    ) -> Result<Vec<AttachmentResponse>>
    where
        I: Into<String>,
    {
        let mut form = reqwest::blocking::multipart::Form::new();

        for (filename, content) in files {
            let part =
                reqwest::blocking::multipart::Part::bytes(content).file_name(filename.to_string());
            form = form.part("file", part);
        }

        self.jira.post_multipart(
            "api",
            &format!("/issue/{}/attachments", issue_key.into()),
            form,
        )
    }
}

/// Provides an iterator over multiple pages of search results
#[derive(Debug)]
pub struct IssuesIter<'a> {
    jira: Jira,
    board: &'a Board,
    results: IssueResults,
    search_options: &'a SearchOptions,
}

impl<'a> IssuesIter<'a> {
    fn new(board: &'a Board, options: &'a SearchOptions, jira: &Jira) -> Result<Self> {
        let results = jira.issues().list(board, options)?;
        Ok(IssuesIter {
            board,
            jira: jira.clone(),
            results,
            search_options: options,
        })
    }

    fn more(&self) -> bool {
        (self.results.start_at + self.results.max_results) <= self.results.total
    }
}

impl Iterator for IssuesIter<'_> {
    type Item = Issue;
    fn next(&mut self) -> Option<Issue> {
        self.results.issues.pop().or_else(|| {
            if self.more() {
                match self.jira.issues().list(
                    self.board,
                    &self
                        .search_options
                        .as_builder()
                        .max_results(self.results.max_results)
                        .start_at(self.results.start_at + self.results.max_results)
                        .build(),
                ) {
                    Ok(new_results) => {
                        self.results = new_results;
                        self.results.issues.pop()
                    }
                    Err(e) => {
                        tracing::error!("Issues pagination failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}

#[cfg(feature = "async")]
/// Async version of the Issues interface
#[derive(Debug)]
pub struct AsyncIssues {
    jira: crate::r#async::Jira,
}

#[cfg(feature = "async")]
impl AsyncIssues {
    pub fn new(jira: &crate::r#async::Jira) -> Self {
        AsyncIssues { jira: jira.clone() }
    }

    /// Get a single issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/issue)
    /// for more information
    pub async fn get<I>(&self, id: I) -> Result<Issue>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/issue/{}", id.into())).await
    }

    /// Get a single issue with custom options
    ///
    /// This method allows you to specify which fields to retrieve, what to expand, and other options.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira};
    /// use gouqi::issues::IssueGetOptions;
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous)?;
    /// let options = IssueGetOptions::builder()
    ///     .fields(vec!["summary".to_string(), "status".to_string()])
    ///     .expand(vec!["changelog".to_string()])
    ///     .build();
    /// let issue = jira.issues().get_with_options("ISSUE-123", &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_with_options<I>(&self, id: I, options: &IssueGetOptions) -> Result<Issue>
    where
        I: Into<String>,
    {
        let url = if let Some(query) = options.to_query_string() {
            format!("/issue/{}?{}", id.into(), query)
        } else {
            format!("/issue/{}", id.into())
        };
        self.jira.get("api", &url).await
    }

    /// Create a new issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-createIssue)
    /// for more information
    pub async fn create(&self, data: CreateIssue) -> Result<CreateResponse> {
        self.jira.post("api", "/issue", data).await
    }

    /// Update an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub async fn update<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira
            .put("api", &format!("/issue/{}", id.into()), data)
            .await
    }

    /// Update an issue with options (async)
    ///
    /// This method allows fine-grained control over the update operation, including
    /// disabling notifications, overriding security settings, and controlling response behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use gouqi::{Credentials, r#async::Jira};
    /// # use gouqi::issues::{EditIssue, IssueUpdateOptions};
    /// # use std::collections::BTreeMap;
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous)?;
    /// // Update without sending notifications
    /// let mut fields = BTreeMap::new();
    /// fields.insert("summary".to_string(), serde_json::Value::String("New summary".to_string()));
    /// let edit = EditIssue { fields };
    ///
    /// let options = IssueUpdateOptions::builder()
    ///     .notify_users(false)
    ///     .build();
    ///
    /// jira.issues().update_with_options("PROJ-123", edit, &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub async fn update_with_options<I, T>(
        &self,
        id: I,
        data: EditIssue<T>,
        options: &IssueUpdateOptions,
    ) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}", id.into())
        } else {
            format!("/issue/{}?{}", id.into(), query_string)
        };
        self.jira.put("api", &path, data).await
    }

    /// Update an issue and return the updated issue (async)
    ///
    /// This method updates an issue and returns the updated issue data in the response.
    /// This is useful when you need to see the result of the update immediately.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use gouqi::{Credentials, r#async::Jira};
    /// # use gouqi::issues::{EditIssue, IssueUpdateOptions};
    /// # use std::collections::BTreeMap;
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous)?;
    /// // Update and get the updated issue back
    /// let mut fields = BTreeMap::new();
    /// fields.insert("summary".to_string(), serde_json::Value::String("Updated summary".to_string()));
    /// let edit = EditIssue { fields };
    ///
    /// let options = IssueUpdateOptions::builder()
    ///     .notify_users(false)
    ///     .return_issue(true)
    ///     .expand(vec!["changelog".to_string()])
    ///     .build();
    ///
    /// let updated_issue = jira.issues().update_and_return("PROJ-123", edit, &options).await?;
    /// println!("Updated: {}", updated_issue.key);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub async fn update_and_return<I, T>(
        &self,
        id: I,
        data: EditIssue<T>,
        options: &IssueUpdateOptions,
    ) -> Result<Issue>
    where
        I: Into<String>,
        T: Serialize,
    {
        let mut opts = options.clone();
        opts.return_issue = true; // Ensure return_issue is set

        let query_string = opts.to_query_string();
        let path = format!("/issue/{}?{}", id.into(), query_string);
        self.jira.put("api", &path, data).await
    }

    /// Edit an issue
    ///
    /// # Deprecated
    ///
    /// Use [`AsyncIssues::update`] instead. This method will be removed in a future version.
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    #[deprecated(
        since = "0.16.0",
        note = "Use `update` instead for consistency with REST conventions"
    )]
    pub async fn edit<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.update(id, data).await
    }

    /// Returns a single page of issue results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard)
    /// for more information
    pub async fn list(&self, board: &Board, options: &SearchOptions) -> Result<IssueResults> {
        let mut path = vec![format!("/board/{}/issue", board.id)];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<IssueResults>("agile", path.join("?").as_ref())
            .await
    }

    /// Return a stream which yields issues from consecutive pages of results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard)
    /// for more information
    pub async fn stream<'a>(
        &'a self,
        board: &'a Board,
        options: &'a SearchOptions,
    ) -> Result<impl Stream<Item = Issue> + 'a> {
        let initial_results = self.list(board, options).await?;

        let stream = AsyncIssuesStream {
            jira: self,
            board,
            search_options: options,
            current_results: initial_results,
            current_index: 0,
        };

        Ok(stream)
    }

    /// Add a comment to an issue (async)
    ///
    /// Automatically detects whether to use V2 (plain text) or V3 (ADF) format
    /// based on the Jira deployment type. For Jira Cloud, uses V3/ADF format.
    /// For Server/Data Center, uses V2/plain text format.
    ///
    /// See [V2 docs](https://developer.atlassian.com/server/jira/platform/jira-rest-api-example-add-comment-8946422/)
    /// and [V3 docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-comments/)
    /// for more information
    pub async fn comment<K>(&self, key: K, data: AddComment) -> Result<Comment>
    where
        K: Into<String>,
    {
        use crate::core::SearchApiVersion;

        let issue_key = key.into();

        // Detect API version (same logic as search API)
        match self.jira.core.get_search_api_version() {
            SearchApiVersion::V3 => {
                // V3 API requires ADF format
                let adf_comment = if let Some(visibility) = data.visibility {
                    AddCommentAdf::from_text(data.body).with_visibility(visibility)
                } else {
                    AddCommentAdf::from_text(data.body)
                };

                self.jira
                    .post_versioned(
                        "api",
                        Some("3"),
                        format!("/issue/{}/comment", issue_key).as_ref(),
                        adf_comment,
                    )
                    .await
            }
            _ => {
                // V2 API uses plain text
                self.jira
                    .post_versioned(
                        "api",
                        Some("latest"),
                        format!("/issue/{}/comment", issue_key).as_ref(),
                        data,
                    )
                    .await
            }
        }
    }

    pub async fn changelog<K>(&self, key: K) -> Result<Changelog>
    where
        K: Into<String>,
    {
        self.jira
            .get("api", format!("/issue/{}/changelog", key.into()).as_ref())
            .await
    }

    /// Extract relationship graph from Jira to specified depth (async version)
    ///
    /// This method asynchronously traverses issue relationships breadth-first starting from
    /// the root issue and builds a declarative relationship graph that can be
    /// used for analysis or applied to other Jira instances.
    ///
    /// # Arguments
    ///
    /// * `root_issue` - The issue key to start traversal from
    /// * `depth` - Maximum depth to traverse (0 = root issue only)
    /// * `options` - Optional configuration for graph extraction
    ///
    /// # Returns
    ///
    /// A `RelationshipGraph` containing all discovered relationships
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use gouqi::{Credentials, r#async::Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous)?;
    /// // Get all relationships 2 levels deep from PROJ-123
    /// let graph = jira.issues()
    ///     .get_relationship_graph("PROJ-123", 2, None).await?;
    ///
    /// // Get only blocking relationships
    /// use gouqi::relationships::GraphOptions;
    /// let options = GraphOptions {
    ///     include_types: Some(vec!["blocks".to_string(), "blocked_by".to_string()]),
    ///     ..Default::default()
    /// };
    /// let blocking_graph = jira.issues()
    ///     .get_relationship_graph("PROJ-123", 1, Some(options)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_relationship_graph(
        &self,
        root_issue: &str,
        depth: u32,
        options: Option<GraphOptions>,
    ) -> Result<RelationshipGraph> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let options = options.unwrap_or_default();
        let mut graph = RelationshipGraph::new("jira_async".to_string());
        graph.metadata.root_issue = Some(root_issue.to_string());
        graph.metadata.max_depth = depth;

        // BFS traversal
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut depth_map = HashMap::new();

        queue.push_back(root_issue.to_string());
        depth_map.insert(root_issue.to_string(), 0);

        while let Some(current_issue) = queue.pop_front() {
            let current_depth = depth_map[&current_issue];

            if visited.contains(&current_issue) {
                continue;
            }
            visited.insert(current_issue.clone());

            // Get the issue details asynchronously
            let issue = match self.get(&current_issue).await {
                Ok(issue) => issue,
                Err(_) => {
                    // Issue not found or not accessible, skip
                    continue;
                }
            };

            // Extract relationships from the issue
            let relationships = self.extract_relationships_from_issue(&issue, &options)?;

            // Add to graph
            graph.add_issue(current_issue.clone(), relationships.clone());

            // If we haven't reached max depth, add related issues to queue
            if current_depth < depth {
                let related_issues = relationships.get_all_related();
                for related_issue in related_issues {
                    if !depth_map.contains_key(&related_issue) {
                        depth_map.insert(related_issue.clone(), current_depth + 1);
                        queue.push_back(related_issue);
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Extract relationships from a single issue (async helper)
    fn extract_relationships_from_issue(
        &self,
        issue: &Issue,
        options: &GraphOptions,
    ) -> Result<IssueRelationships> {
        let mut relationships = IssueRelationships::new();

        // Extract issue links
        if let Some(Ok(links)) = issue.links() {
            for link in links {
                let link_type_name = &link.link_type.name;

                // Check if this link type should be included
                if let Some(ref include_types) = options.include_types {
                    if !include_types.contains(link_type_name) {
                        continue;
                    }
                }
                if let Some(ref exclude_types) = options.exclude_types {
                    if exclude_types.contains(link_type_name) {
                        continue;
                    }
                }

                // Map Jira link types to our standard types
                let (outward_type, inward_type) = self.map_link_type(link_type_name);

                // Add outward relationship
                if let Some(ref outward_issue) = link.outward_issue {
                    if options.bidirectional || Some(&issue.key) != Some(&outward_issue.key) {
                        relationships.add_relationship(&outward_type, outward_issue.key.clone());
                    }
                }

                // Add inward relationship
                if let Some(ref inward_issue) = link.inward_issue {
                    if options.bidirectional || Some(&issue.key) != Some(&inward_issue.key) {
                        relationships.add_relationship(&inward_type, inward_issue.key.clone());
                    }
                }

                // Add to custom if not a standard type
                if !self.is_standard_link_type(link_type_name) && options.include_custom {
                    if let Some(ref outward_issue) = link.outward_issue {
                        relationships.add_relationship(
                            &format!("custom_{}", link_type_name.to_lowercase()),
                            outward_issue.key.clone(),
                        );
                    }
                    if let Some(ref inward_issue) = link.inward_issue {
                        relationships.add_relationship(
                            &format!("custom_{}_inward", link_type_name.to_lowercase()),
                            inward_issue.key.clone(),
                        );
                    }
                }
            }
        }

        // Extract parent-child relationships
        if let Some(parent_issue) = issue.parent() {
            relationships.parent = Some(parent_issue.key);
        }

        // Extract epic relationships (if available in custom fields)
        if let Some(epic_link) = self.extract_epic_link(issue) {
            relationships.epic = Some(epic_link);
        }

        Ok(relationships)
    }

    /// Map Jira link type names to our standard relationship types (async helper)
    fn map_link_type(&self, link_type_name: &str) -> (String, String) {
        match link_type_name.to_lowercase().as_str() {
            "blocks" => ("blocks".to_string(), "blocked_by".to_string()),
            "duplicate" | "duplicates" => ("duplicates".to_string(), "duplicated_by".to_string()),
            "relates" | "relates to" => ("relates_to".to_string(), "relates_to".to_string()),
            "clones" => ("duplicates".to_string(), "duplicated_by".to_string()),
            "causes" => ("blocks".to_string(), "blocked_by".to_string()),
            _ => (
                format!("custom_{}", link_type_name.to_lowercase()),
                format!("custom_{}_inward", link_type_name.to_lowercase()),
            ),
        }
    }

    /// Check if a link type is one of our standard types (async helper)
    fn is_standard_link_type(&self, link_type_name: &str) -> bool {
        matches!(
            link_type_name.to_lowercase().as_str(),
            "blocks" | "duplicate" | "duplicates" | "relates" | "relates to" | "clones" | "causes"
        )
    }

    /// Extract epic link from issue (async helper)
    fn extract_epic_link(&self, issue: &Issue) -> Option<String> {
        // This is a common custom field for Epic Link
        // You may need to adjust the field name based on your Jira configuration
        issue
            .field::<String>("customfield_10014")
            .and_then(|result| result.ok())
            .or_else(|| {
                issue
                    .field::<String>("customfield_10008")
                    .and_then(|result| result.ok())
            })
            .or_else(|| {
                issue
                    .field::<String>("Epic Link")
                    .and_then(|result| result.ok())
            })
    }

    /// Get current relationships for multiple issues efficiently (async version)
    ///
    /// This is more efficient than calling `get_relationship_graph` for each issue
    /// individually when you need relationships for a known set of issues.
    ///
    /// # Arguments
    ///
    /// * `issue_keys` - List of issue keys to get relationships for
    /// * `options` - Optional configuration for relationship extraction
    ///
    /// # Returns
    ///
    /// A `RelationshipGraph` containing relationships for all specified issues
    pub async fn get_bulk_relationships(
        &self,
        issue_keys: &[String],
        options: Option<GraphOptions>,
    ) -> Result<RelationshipGraph> {
        let options = options.unwrap_or_default();
        let mut graph = RelationshipGraph::new("jira_bulk_async".to_string());
        graph.metadata.max_depth = 0; // Direct relationships only

        // Process issues sequentially for now (can be optimized later)
        for issue_key in issue_keys {
            match self.get(issue_key).await {
                Ok(issue) => {
                    let relationships = self.extract_relationships_from_issue(&issue, &options)?;
                    graph.add_issue(issue_key.clone(), relationships);
                }
                Err(_) => {
                    // Issue not found or not accessible, skip but could log
                    continue;
                }
            }
        }

        Ok(graph)
    }

    /// Delete an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-deleteIssue)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the issue cannot be deleted due to workflow restrictions
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/issue/{}", id.into()))
            .await?;
        Ok(())
    }

    /// Delete an issue with options (async)
    ///
    /// Deletes an issue from Jira with additional options such as deleting subtasks.
    ///
    /// # Arguments
    ///
    /// * `id` - The issue key (e.g., "PROJ-123") or ID
    /// * `options` - Options for the delete operation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira};
    /// use gouqi::issues::IssueDeleteOptions;
    ///
    /// let jira = Jira::new("https://jira.example.com", Credentials::Anonymous)?;
    /// let options = IssueDeleteOptions::builder()
    ///     .delete_subtasks(true)
    ///     .build();
    /// jira.issues().delete_with_options("PROJ-123", options).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The issue does not exist
    /// - The user lacks permission to delete the issue
    /// - The issue cannot be deleted due to workflow restrictions
    ///
    /// # Panics
    ///
    /// This function will panic if the issue cannot be deleted due to workflow restrictions
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-deleteIssue)
    /// for more information
    pub async fn delete_with_options<I>(&self, id: I, options: IssueDeleteOptions) -> Result<()>
    where
        I: Into<String>,
    {
        let query = options.to_query_string();
        let path = if query.is_empty() {
            format!("/issue/{}", id.into())
        } else {
            format!("/issue/{}?{}", id.into(), query)
        };
        self.jira
            .delete::<crate::EmptyResponse>("api", &path)
            .await?;
        Ok(())
    }

    /// Archive an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-archiveIssue)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the issue cannot be archived
    pub async fn archive<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/archive", id.into()), ())
            .await
    }

    /// Get all worklogs for an issue (async)
    pub async fn get_worklogs<K>(&self, issue_key: K) -> Result<crate::WorklogList>
    where
        K: Into<String>,
    {
        self.jira
            .get("api", &format!("/issue/{}/worklog", issue_key.into()))
            .await
    }

    /// Get a specific worklog by ID (async)
    pub async fn get_worklog<K, W>(&self, issue_key: K, worklog_id: W) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira
            .get(
                "api",
                &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
            )
            .await
    }

    /// Add a worklog to an issue (async)
    pub async fn add_worklog<K>(
        &self,
        issue_key: K,
        worklog: crate::WorklogInput,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
    {
        self.jira
            .post(
                "api",
                &format!("/issue/{}/worklog", issue_key.into()),
                worklog,
            )
            .await
    }

    /// Update an existing worklog (async)
    pub async fn update_worklog<K, W>(
        &self,
        issue_key: K,
        worklog_id: W,
        worklog: crate::WorklogInput,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira
            .put(
                "api",
                &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
                worklog,
            )
            .await
    }

    /// Add a worklog with options for time tracking and notifications (async)
    ///
    /// This method allows you to control how the remaining estimate is adjusted when logging work,
    /// and whether to send notifications to watchers.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog` - The worklog data to add
    /// * `options` - Options controlling estimate adjustment and notifications
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira, WorklogInput};
    /// use gouqi::issues::{WorklogOptions, AdjustEstimate};
    ///
    /// let jira = Jira::new("http://localhost", Credentials::Anonymous)?;
    /// // Log work and set a new remaining estimate
    /// let worklog = WorklogInput::new(7200).with_comment("Implemented feature");
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::New("1d".to_string()))
    ///     .notify_users(false)
    ///     .build();
    ///
    /// let created = jira.issues().add_worklog_with_options("PROJ-123", worklog, &options).await?;
    /// println!("Created worklog: {}", created.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_worklog_with_options<K>(
        &self,
        issue_key: K,
        worklog: crate::WorklogInput,
        options: &WorklogOptions,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}/worklog", issue_key.into())
        } else {
            format!("/issue/{}/worklog?{}", issue_key.into(), query_string)
        };
        self.jira.post("api", &path, worklog).await
    }

    /// Update a worklog with options for time tracking and notifications (async)
    ///
    /// This method allows you to control how the remaining estimate is adjusted when updating work,
    /// and whether to send notifications to watchers.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123") or ID
    /// * `worklog_id` - The ID of the worklog to update
    /// * `worklog` - The updated worklog data
    /// * `options` - Options controlling estimate adjustment and notifications
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira, WorklogInput};
    /// use gouqi::issues::{WorklogOptions, AdjustEstimate};
    ///
    /// let jira = Jira::new("http://localhost", Credentials::Anonymous)?;
    /// // Update work and reduce the estimate by a specific amount
    /// let worklog = WorklogInput::new(3600).with_comment("Updated time");
    /// let options = WorklogOptions::builder()
    ///     .adjust_estimate(AdjustEstimate::Manual("30m".to_string()))
    ///     .notify_users(false)
    ///     .build();
    ///
    /// let updated = jira.issues().update_worklog_with_options("PROJ-123", "10001", worklog, &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_worklog_with_options<K, W>(
        &self,
        issue_key: K,
        worklog_id: W,
        worklog: crate::WorklogInput,
        options: &WorklogOptions,
    ) -> Result<crate::Worklog>
    where
        K: Into<String>,
        W: Into<String>,
    {
        let query_string = options.to_query_string();
        let path = if query_string.is_empty() {
            format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into())
        } else {
            format!(
                "/issue/{}/worklog/{}?{}",
                issue_key.into(),
                worklog_id.into(),
                query_string
            )
        };
        self.jira.put("api", &path, worklog).await
    }

    /// Delete a worklog (async)
    pub async fn delete_worklog<K, W>(&self, issue_key: K, worklog_id: W) -> Result<()>
    where
        K: Into<String>,
        W: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>(
                "api",
                &format!("/issue/{}/worklog/{}", issue_key.into(), worklog_id.into()),
            )
            .await?;
        Ok(())
    }

    /// Assign an issue to a user (async, None for unassign)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-assign)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the assignee is invalid
    pub async fn assign<I>(&self, id: I, assignee: Option<String>) -> Result<()>
    where
        I: Into<String>,
    {
        let assign_request = AssignRequest { assignee };
        self.jira
            .put(
                "api",
                &format!("/issue/{}/assignee", id.into()),
                assign_request,
            )
            .await
    }

    /// Get list of users watching an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-getIssueWatchers)
    /// for more information
    pub async fn get_watchers<I>(&self, id: I) -> Result<Watchers>
    where
        I: Into<String>,
    {
        self.jira
            .get("api", &format!("/issue/{}/watchers", id.into()))
            .await
    }

    /// Add a watcher to an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-addWatcher)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the user cannot be added as a watcher
    pub async fn add_watcher<I>(&self, id: I, username: String) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/watchers", id.into()), username)
            .await
    }

    /// Remove a watcher from an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-removeWatcher)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if the watcher cannot be removed
    pub async fn remove_watcher<I>(&self, id: I, username: String) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete(
                "api",
                &format!("/issue/{}/watchers?username={}", id.into(), username),
            )
            .await
    }

    /// Vote for an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-addVote)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if voting fails
    pub async fn vote<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .post("api", &format!("/issue/{}/votes", id.into()), ())
            .await
    }

    /// Remove vote from an issue (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-removeVote)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if vote removal fails
    pub async fn unvote<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete("api", &format!("/issue/{}/votes", id.into()))
            .await
    }

    /// Create multiple issues in a single request (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-createIssues)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if any issue creation fails validation
    pub async fn bulk_create(&self, issues: Vec<CreateIssue>) -> Result<BulkCreateResponse> {
        let bulk_request = BulkCreateRequest {
            issue_updates: issues,
        };
        self.jira.post("api", "/issue/bulk", bulk_request).await
    }

    /// Update multiple issues in a single request (async)
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/issue-updateBulkIssues)
    /// for more information
    ///
    /// # Panics
    ///
    /// This function will panic if any update fails validation
    pub async fn bulk_update(&self, updates: BulkUpdateRequest) -> Result<BulkUpdateResponse> {
        self.jira.put("api", "/issue/bulk", updates).await
    }

    /// Upload one or more attachments to an issue (async)
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The issue key (e.g., "PROJ-123")
    /// * `files` - Vector of tuples containing (filename, file_content)
    ///
    /// # Returns
    ///
    /// `Result<Vec<AttachmentResponse>>` - Array of uploaded attachment metadata
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira};
    ///
    /// let jira = Jira::new("https://example.atlassian.net", Credentials::Anonymous)?;
    /// let file_content = std::fs::read("document.pdf")?;
    /// let attachments = jira.issues()
    ///     .upload_attachment("PROJ-123", vec![("document.pdf", file_content)])
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_attachment<I>(
        &self,
        issue_key: I,
        files: Vec<(&str, Vec<u8>)>,
    ) -> Result<Vec<AttachmentResponse>>
    where
        I: Into<String>,
    {
        let mut form = reqwest::multipart::Form::new();

        for (filename, content) in files {
            let part = reqwest::multipart::Part::bytes(content).file_name(filename.to_string());
            form = form.part("file", part);
        }

        self.jira
            .post_multipart(
                "api",
                &format!("/issue/{}/attachments", issue_key.into()),
                form,
            )
            .await
    }
}

#[cfg(feature = "async")]
struct AsyncIssuesStream<'a> {
    jira: &'a AsyncIssues,
    board: &'a Board,
    search_options: &'a SearchOptions,
    current_results: IssueResults,
    current_index: usize,
}

#[cfg(feature = "async")]
impl Stream for AsyncIssuesStream<'_> {
    type Item = Issue;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;

        // If we still have issues in the current page
        if self.current_index < self.current_results.issues.len() {
            let issue = self.current_results.issues[self.current_index].clone();
            self.current_index += 1;
            return Poll::Ready(Some(issue));
        }

        // Check if we need to fetch the next page
        let more_pages = (self.current_results.start_at + self.current_results.max_results)
            <= self.current_results.total;

        if more_pages {
            // Create a future to fetch the next page
            let jira = self.jira;
            let board = self.board;
            let next_options = self
                .search_options
                .as_builder()
                .max_results(self.current_results.max_results)
                .start_at(self.current_results.start_at + self.current_results.max_results)
                .build();

            let future = async move { jira.list(board, &next_options).await };

            // Poll the future
            let mut future = Box::pin(future);
            match future.as_mut().poll(cx) {
                Poll::Ready(Ok(new_results)) => {
                    // No results in the new page
                    if new_results.issues.is_empty() {
                        return Poll::Ready(None);
                    }

                    // Update state with new results
                    self.current_results = new_results;
                    self.current_index = 0;

                    // Return the first issue from the new page
                    let issue = self.current_results.issues[0].clone();
                    self.current_index = 1;
                    Poll::Ready(Some(issue))
                }
                Poll::Ready(Err(_)) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        } else {
            // No more pages
            Poll::Ready(None)
        }
    }
}
