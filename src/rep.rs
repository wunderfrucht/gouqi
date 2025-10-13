// Third party

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use tracing::error;

// Ours
use crate::{Jira, Result};
// Forward reference - Component is defined later in this file but used by Project

/// Custom serde module for JIRA datetime format
/// JIRA requires dates in format: "2024-01-01T09:00:00.000+0000"
/// This differs from standard ISO8601 in three ways:
/// 1. No year sign prefix (2024 not +002024)
/// 2. Milliseconds not nanoseconds (.000 not .000000000)
/// 3. Timezone as +0000 not Z
pub(crate) mod jira_datetime {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

    /// Serializes OffsetDateTime to JIRA's expected format
    pub fn serialize<S>(dt: &Option<OffsetDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => {
                // Format as: "2024-01-01T09:00:00.000+0000"
                let format = time::format_description::parse(
                    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory][offset_minute]"
                ).map_err(serde::ser::Error::custom)?;

                let formatted = dt.format(&format).map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(&formatted)
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from JIRA datetime format or standard ISO8601
    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer)?
            .map(|s| {
                // Try JIRA format first, then fall back to standard ISO8601
                let format = time::format_description::parse(
                    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory][offset_minute]"
                ).map_err(|e| serde::de::Error::custom(format!("Format parse error: {}", e)))?;

                OffsetDateTime::parse(&s, &format)
                    .or_else(|_| OffsetDateTime::parse(&s, &time::format_description::well_known::Iso8601::DEFAULT))
                    .map_err(|e| serde::de::Error::custom(format!("Date parse error: {}", e)))
            })
            .transpose()
    }
}

/// Represents an general jira error response
#[derive(Serialize, Deserialize, Debug)]
pub struct Errors {
    #[serde(rename = "errorMessages", default)]
    pub error_messages: Vec<String>,
    #[serde(default)]
    pub errors: BTreeMap<String, String>,
    // Support for V3 API error format
    #[serde(default)]
    pub error: Option<String>,
}

/// Represents a single jira issue
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issue {
    #[serde(rename = "self")]
    pub self_link: String,
    pub key: String,
    pub id: String,
    pub fields: BTreeMap<String, ::serde_json::Value>,
}

impl Issue {
    /// Resolves a typed field from an issues lists of arbitrary fields
    pub fn field<F>(&self, name: &str) -> Option<Result<F>>
    where
        for<'de> F: Deserialize<'de>,
    {
        self.fields
            .get(name)
            .map(|value| Ok(serde_json::value::from_value::<F>(value.clone())?))
    }

    fn user_field(&self, name: &str) -> Option<Result<User>> {
        self.field::<User>(name)
    }

    fn string_field(&self, name: &str) -> Option<Result<String>> {
        self.field::<String>(name)
    }

    /// User assigned to issue
    pub fn assignee(&self) -> Option<User> {
        self.user_field("assignee").and_then(|value| value.ok())
    }

    /// User that created the issue
    pub fn creator(&self) -> Option<User> {
        self.user_field("creator").and_then(|value| value.ok())
    }

    /// User that reported the issue
    pub fn reporter(&self) -> Option<User> {
        self.user_field("reporter").and_then(|value| value.ok())
    }

    /// The current status of the issue
    pub fn status(&self) -> Option<Status> {
        self.field::<Status>("status").and_then(|value| value.ok())
    }

    /// Brief summary of the issue
    pub fn summary(&self) -> Option<String> {
        self.string_field("summary").and_then(|value| value.ok())
    }

    /// Description of the issue
    ///
    /// Supports both legacy string format (JIRA v2) and Atlassian Document Format (JIRA v3).
    /// For ADF format, converts the structured document to plain text.
    pub fn description(&self) -> Option<String> {
        // First try to get as string (legacy v2 API format)
        if let Some(Ok(desc)) = self.string_field("description") {
            return Some(desc);
        }

        // If that fails, try to parse as ADF (v3 API format)
        if let Some(Ok(adf)) = self.field::<AdfDocument>("description") {
            let plain_text = adf.to_plain_text();
            return if plain_text.is_empty() {
                None
            } else {
                Some(plain_text)
            };
        }

        None
    }

    /// Environment information for the issue
    ///
    /// Supports both legacy string format (JIRA v2) and Atlassian Document Format (JIRA v3).
    /// For ADF format, converts the structured document to plain text.
    pub fn environment(&self) -> Option<String> {
        // First try to get as string (legacy v2 API format)
        if let Some(Ok(env)) = self.string_field("environment") {
            return Some(env);
        }

        // If that fails, try to parse as ADF (v3 API format)
        if let Some(Ok(adf)) = self.field::<AdfDocument>("environment") {
            let plain_text = adf.to_plain_text();
            return if plain_text.is_empty() {
                None
            } else {
                Some(plain_text)
            };
        }

        None
    }

    fn extract_offset_date_time(&self, field: &str) -> Option<OffsetDateTime> {
        match self.string_field(field) {
            Some(Ok(created)) => match OffsetDateTime::parse(created.as_ref(), &Iso8601::DEFAULT) {
                Ok(offset_date_time) => Some(offset_date_time),
                Err(error) => {
                    error!(
                        "Can't convert '{} = {:?}' into a OffsetDateTime. {:?}",
                        field, created, error
                    );
                    None
                }
            },
            _ => None,
        }
    }

    /// Updated timestamp
    pub fn updated(&self) -> Option<OffsetDateTime> {
        self.extract_offset_date_time("updated")
    }

    /// Created timestamp
    pub fn created(&self) -> Option<OffsetDateTime> {
        self.extract_offset_date_time("created")
    }

    pub fn resolution_date(&self) -> Option<OffsetDateTime> {
        self.extract_offset_date_time("resolutiondate")
    }

    /// An issue type
    pub fn issue_type(&self) -> Option<IssueType> {
        self.field::<IssueType>("issuetype")
            .and_then(|value| value.ok())
    }

    /// Labels associated with the issue
    pub fn labels(&self) -> Vec<String> {
        self.field::<Vec<String>>("labels")
            .and_then(|value| value.ok())
            .unwrap_or_default()
    }

    /// List of versions associated with the issue
    pub fn fix_versions(&self) -> Vec<Version> {
        self.field::<Vec<Version>>("fixVersions")
            .and_then(|value| value.ok())
            .unwrap_or_default()
    }

    /// Priority of the issue
    pub fn priority(&self) -> Option<Priority> {
        self.field::<Priority>("priority")
            .and_then(|value| value.ok())
    }

    /// Links to other issues
    pub fn links(&self) -> Option<Result<Vec<IssueLink>>> {
        self.field::<Vec<IssueLink>>("issuelinks") //.and_then(|value| value.ok()).unwrap_or(vec![])
    }

    pub fn project(&self) -> Option<Project> {
        self.field::<Project>("project")
            .and_then(|value| value.ok())
    }

    pub fn resolution(&self) -> Option<Resolution> {
        self.field::<Resolution>("resolution")
            .and_then(|value| value.ok())
    }

    pub fn attachment(&self) -> Vec<Attachment> {
        self.field::<Vec<Attachment>>("attachment")
            .and_then(|value| value.ok())
            .unwrap_or_default()
    }

    pub fn comments(&self) -> Option<Comments> {
        self.field::<Comments>("comment")
            .and_then(|value| value.ok())
    }

    pub fn parent(&self) -> Option<Issue> {
        self.field::<Issue>("parent").and_then(|value| value.ok())
    }

    pub fn timetracking(&self) -> Option<TimeTracking> {
        self.field::<TimeTracking>("timetracking")
            .and_then(|value| value.ok())
    }

    /// Returns a permanent link to the issue in the Jira web interface
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The host URL cannot be joined with the browse path
    /// - The issue key cannot be added to the URL path
    pub fn permalink(&self, jira: &Jira) -> String {
        //format!("{}/browse/{}", jira.host, self.key)
        jira.host()
            .join("/browse/")
            .unwrap()
            .join(&self.key)
            .unwrap()
            .to_string()
    }

    pub fn try_from_custom_issue<S: Serialize>(custom_issue: &S) -> serde_json::Result<Self> {
        let serialized_data = serde_json::to_string(custom_issue)?;
        serde_json::from_str(&serialized_data)
    }

    pub fn try_to_custom_issue<D: DeserializeOwned>(&self) -> serde_json::Result<D> {
        let serialized_data = serde_json::to_string(self)?;
        serde_json::from_str(&serialized_data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub filename: String,
    pub author: User,
    pub created: String,
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub content: String,
    pub thumbnail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    pub comments: Vec<Comment>,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,
    #[serde(rename = "startAt")]
    pub start_at: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub id: Option<String>,
    #[serde(rename = "self")]
    pub self_link: String,
    pub author: Option<User>,
    #[serde(rename = "updateAuthor")]
    pub update_author: Option<User>,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub created: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub updated: Option<OffsetDateTime>,
    pub body: TextContent,
    pub visibility: Option<Visibility>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Visibility {
    #[serde(rename = "type")]
    pub visibility_type: String,
    pub value: String,
}

/// Text content that supports both plain text (JIRA v2) and ADF format (JIRA v3)
///
/// This type provides backward compatibility by accepting both String and ADF JSON
/// during deserialization, while providing string-like access patterns through Deref.
///
/// # Examples
///
/// ```
/// # use gouqi::TextContent;
/// // Acts like a string reference
/// let text = TextContent::from("Hello");
/// assert_eq!(text.len(), 5);
/// assert!(text.contains("Hello"));
/// ```
#[derive(Clone, Debug)]
pub struct TextContent {
    /// Raw JSON value (either String or ADF document)
    raw: serde_json::Value,
    /// Cached extracted text for efficient access
    cached: String,
}

impl TextContent {
    /// Create TextContent from a plain string
    pub fn from_string(s: impl Into<String>) -> Self {
        let text = s.into();
        Self {
            raw: serde_json::Value::String(text.clone()),
            cached: text,
        }
    }

    /// Get the raw JSON value
    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }

    /// Extract plain text from a JSON value (String or ADF)
    fn extract_text(value: &serde_json::Value) -> String {
        // First try to get as string (legacy v2 API format)
        if let Ok(text) = serde_json::from_value::<String>(value.clone()) {
            return text;
        }

        // If that fails, try to parse as ADF (v3 API format)
        if let Ok(adf) = serde_json::from_value::<AdfDocument>(value.clone()) {
            return adf.to_plain_text();
        }

        // Fallback: empty string
        String::new()
    }
}

impl std::ops::Deref for TextContent {
    type Target = str;

    fn deref(&self) -> &str {
        &self.cached
    }
}

impl std::fmt::Display for TextContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cached)
    }
}

impl AsRef<str> for TextContent {
    fn as_ref(&self) -> &str {
        &self.cached
    }
}

impl std::borrow::Borrow<str> for TextContent {
    fn borrow(&self) -> &str {
        &self.cached
    }
}

impl PartialEq<str> for TextContent {
    fn eq(&self, other: &str) -> bool {
        self.cached == other
    }
}

impl PartialEq<&str> for TextContent {
    fn eq(&self, other: &&str) -> bool {
        self.cached == *other
    }
}

impl PartialEq<String> for TextContent {
    fn eq(&self, other: &String) -> bool {
        &self.cached == other
    }
}

impl PartialEq for TextContent {
    fn eq(&self, other: &Self) -> bool {
        self.cached == other.cached
    }
}

impl Eq for TextContent {}

impl From<String> for TextContent {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for TextContent {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

// Custom serialization: always serialize the raw value
impl Serialize for TextContent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.raw.serialize(serializer)
    }
}

// Custom deserialization: accept both String and ADF, eagerly extract text
impl<'de> Deserialize<'de> for TextContent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        let cached = Self::extract_text(&raw);
        Ok(Self { raw, cached })
    }
}

/// Atlassian Document Format (ADF) structures for V3 API comments
/// See: https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/
/// ADF text node - inline content with optional formatting
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdfText {
    #[serde(rename = "type")]
    pub node_type: String, // "text"
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<AdfMark>>,
}

impl AdfText {
    /// Create a plain text node
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            node_type: "text".to_string(),
            text: text.into(),
            marks: None,
        }
    }
}

/// ADF text formatting marks (bold, italic, etc.)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdfMark {
    #[serde(rename = "type")]
    pub mark_type: String, // "strong", "em", "code", etc.
}

/// ADF block node - can be paragraph, heading, list, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdfNode {
    #[serde(rename = "type")]
    pub node_type: String, // "paragraph", "heading", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<AdfContent>>,
}

impl AdfNode {
    /// Create a paragraph node containing text
    pub fn paragraph(content: Vec<AdfText>) -> Self {
        Self {
            node_type: "paragraph".to_string(),
            content: Some(content.into_iter().map(AdfContent::Text).collect()),
        }
    }
}

/// ADF content - can be either inline text or nested nodes
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AdfContent {
    Text(AdfText),
    Node(Box<AdfNode>),
}

/// ADF document root structure for V3 API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdfDocument {
    pub version: u32,
    #[serde(rename = "type")]
    pub doc_type: String, // "doc"
    pub content: Vec<AdfNode>,
}

impl AdfDocument {
    /// Create an ADF document from plain text
    /// Splits text by newlines and creates a paragraph for each line
    pub fn from_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let content = if text.is_empty() {
            // Empty document needs at least one empty paragraph
            vec![AdfNode::paragraph(vec![])]
        } else {
            // Split by newlines and create a paragraph for each non-empty line
            text.lines()
                .map(|line| {
                    if line.is_empty() {
                        AdfNode::paragraph(vec![])
                    } else {
                        AdfNode::paragraph(vec![AdfText::new(line)])
                    }
                })
                .collect()
        };

        Self {
            version: 1,
            doc_type: "doc".to_string(),
            content,
        }
    }

    /// Extract plain text from an ADF document
    /// Converts the structured document back to plain text, joining paragraphs with newlines
    pub fn to_plain_text(&self) -> String {
        self.content
            .iter()
            .filter_map(Self::extract_text_from_node)
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Recursively extract text from an ADF node
    fn extract_text_from_node(node: &AdfNode) -> Option<String> {
        if let Some(content) = &node.content {
            let text: Vec<String> = content
                .iter()
                .filter_map(|item| match item {
                    AdfContent::Text(text_node) => Some(text_node.text.clone()),
                    AdfContent::Node(nested_node) => Self::extract_text_from_node(nested_node),
                })
                .collect();

            if text.is_empty() {
                None
            } else {
                Some(text.join(""))
            }
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Changelog {
    #[serde(rename = "values")]
    pub histories: Vec<History>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct History {
    pub author: User,
    pub created: String,
    pub items: Vec<HistoryItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryItem {
    pub field: String,
    pub from: Option<String>,
    #[serde(rename = "fromString")]
    pub from_string: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "toString")]
    pub to_string: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(rename = "self")]
    pub self_link: String,
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub project_type_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ProjectComponent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Vec<Version>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_urls: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_category: Option<ProjectCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_types: Option<Vec<IssueType>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectComponent {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateProject {
    pub key: String,
    pub name: String,
    pub project_type_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<String>, // username or account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_security_scheme: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scheme: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_scheme: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectCategory {
    #[serde(rename = "self")]
    pub self_link: String,
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Debug, Default)]
pub struct ProjectSearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type_key: Option<String>,
}

impl ProjectSearchOptions {
    /// Serialize the search options to query parameters
    pub fn serialize(&self) -> Result<Vec<(String, String)>> {
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
        if let Some(ref order_by) = self.order_by {
            params.push(("orderBy".to_string(), order_by.clone()));
        }
        if let Some(category_id) = self.category_id {
            params.push(("categoryId".to_string(), category_id.to_string()));
        }
        if let Some(ref project_type_key) = self.project_type_key {
            params.push(("projectTypeKey".to_string(), project_type_key.clone()));
        }

        Ok(params)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSearchResults {
    pub start_at: u64,
    pub max_results: u64,
    pub total: u64,
    pub values: Vec<Project>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectRole {
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    pub id: u64,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actors: Option<Vec<RoleActor>>,
}

#[derive(Deserialize, Debug)]
pub struct RoleActor {
    pub id: u64,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Represents link relationship between issues
#[derive(Serialize, Deserialize, Debug)]
pub struct IssueLink {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "outwardIssue")]
    pub outward_issue: Option<Issue>,
    #[serde(rename = "inwardIssue")]
    pub inward_issue: Option<Issue>,
    #[serde(rename = "type")]
    pub link_type: LinkType,
}

/// Represents type of issue relation
#[derive(Serialize, Deserialize, Debug)]
pub struct LinkType {
    pub id: String,
    pub inward: String,
    pub name: String,
    pub outward: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

/// Request to create an issue link
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueLinkInput {
    #[serde(rename = "type")]
    pub link_type: IssueLinkType,
    pub inward_issue: IssueKey,
    pub outward_issue: IssueKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<LinkComment>,
}

/// Simple issue link type reference for creating links
#[derive(Serialize, Debug)]
pub struct IssueLinkType {
    pub name: String,
}

/// Simple issue key reference for creating links
#[derive(Serialize, Debug)]
pub struct IssueKey {
    pub key: String,
}

/// Optional comment when creating an issue link
///
/// Automatically converts plain text to ADF format for v3 API compatibility
#[derive(Debug)]
pub struct LinkComment {
    body: String,
}

impl LinkComment {
    /// Create a new link comment
    pub fn new(body: impl Into<String>) -> Self {
        Self { body: body.into() }
    }
}

impl Serialize for LinkComment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        // Convert plain text to ADF format for v3 API
        let adf = AdfDocument::from_text(&self.body);

        let mut state = serializer.serialize_struct("LinkComment", 1)?;
        state.serialize_field("body", &adf)?;
        state.end()
    }
}

impl CreateIssueLinkInput {
    /// Create a new issue link
    pub fn new(
        link_type: impl Into<String>,
        inward: impl Into<String>,
        outward: impl Into<String>,
    ) -> Self {
        Self {
            link_type: IssueLinkType {
                name: link_type.into(),
            },
            inward_issue: IssueKey { key: inward.into() },
            outward_issue: IssueKey {
                key: outward.into(),
            },
            comment: None,
        }
    }

    /// Add a comment to the issue link
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(LinkComment::new(comment));
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    pub archived: bool,
    pub id: String,
    pub name: String,
    #[serde(rename = "projectId")]
    pub project_id: u64,
    pub released: bool,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Serialize, Debug)]
pub struct VersionCreationBody {
    pub name: String,
    #[serde(rename = "projectId")]
    pub project_id: u64,
}

#[derive(Serialize, Debug)]
pub struct VersionMoveAfterBody {
    pub after: String,
}

#[derive(Serialize, Debug)]
pub struct VersionUpdateBody {
    pub released: bool,
    pub archived: bool,
    #[serde(rename = "moveUnfixedIssuesTo")]
    pub move_unfixed_issues_to: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub active: bool,
    #[serde(rename = "avatarUrls")]
    pub avatar_urls: Option<BTreeMap<String, String>>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "timeZone")]
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub description: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Priority {
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueType {
    pub description: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub subtask: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    /// Total number of issues. Note: V3 API doesn't provide this, so it may be estimated
    pub total: u64,
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    pub expand: Option<String>,
    pub issues: Vec<Issue>,
    /// V3 API specific: Indicates if this is the last page of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_last_page: Option<bool>,
    /// V3 API specific: Token for fetching the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// Indicates if the total count is accurate (false for V3 API)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_is_accurate: Option<bool>,
}

/// V3 Search Results format for the new /rest/api/3/search/jql endpoint
#[derive(Serialize, Deserialize, Debug)]
pub struct V3SearchResults {
    pub issues: Vec<Issue>,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

impl V3SearchResults {
    /// Convert V3SearchResults to legacy SearchResults format for backward compatibility
    pub fn to_search_results(self, start_at: u64, max_results: u64) -> SearchResults {
        // V3 API doesn't provide total count. We provide a best-effort estimate
        // but mark it as inaccurate. This maintains backward compatibility while
        // being honest about the limitation.
        let total = if self.is_last {
            // If this is the last page, we know the exact total
            start_at + self.issues.len() as u64
        } else {
            // If not last page, use a high estimate to indicate more pages exist
            // Using u64::MAX would break existing code, so we estimate conservatively
            // Assume at least one more full page exists
            start_at + self.issues.len() as u64 + max_results
        };

        SearchResults {
            total,
            max_results,
            start_at,
            expand: None,
            issues: self.issues,
            is_last_page: Some(self.is_last),
            next_page_token: self.next_page_token,
            total_is_accurate: Some(false), // V3 never provides accurate totals
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeTracking {
    pub original_estimate: Option<String>,
    pub original_estimate_seconds: Option<u64>,
    pub remaining_estimate: Option<String>,
    pub remaining_estimate_seconds: Option<u64>,
    pub time_spent: Option<String>,
    pub time_spent_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransitionOption {
    pub id: String,
    pub name: String,
    pub to: TransitionTo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransitionTo {
    pub name: String,
    pub id: String,
}

/// Contains list of options an issue can transitions through
#[derive(Serialize, Deserialize, Debug)]
pub struct TransitionOptions {
    pub transitions: Vec<TransitionOption>,
}

#[derive(Serialize, Debug)]
pub struct TransitionTriggerOptions {
    pub transition: Transition,
    pub fields: BTreeMap<String, ::serde_json::Value>,
}

impl TransitionTriggerOptions {
    /// Creates a new instance
    pub fn new<I>(id: I) -> TransitionTriggerOptions
    where
        I: Into<String>,
    {
        TransitionTriggerOptions {
            transition: Transition { id: id.into() },
            fields: BTreeMap::new(),
        }
    }

    pub fn builder<I>(id: I) -> TransitionTriggerOptionsBuilder
    where
        I: Into<String>,
    {
        TransitionTriggerOptionsBuilder::new(id)
    }
}

pub struct TransitionTriggerOptionsBuilder {
    pub transition: Transition,
    pub fields: BTreeMap<String, ::serde_json::Value>,
}

impl TransitionTriggerOptionsBuilder {
    /// Creates a new instance
    pub fn new<I>(id: I) -> TransitionTriggerOptionsBuilder
    where
        I: Into<String>,
    {
        TransitionTriggerOptionsBuilder {
            transition: Transition { id: id.into() },
            fields: BTreeMap::new(),
        }
    }

    /// Appends a field to update as part of transition
    ///
    /// # Panics
    ///
    /// This function will panic if the provided value cannot be serialized to JSON.
    /// This should only happen in exceptional circumstances, such as when a custom type
    /// with a failing serialization implementation is provided.
    pub fn field<N, V>(&mut self, name: N, value: V) -> &mut TransitionTriggerOptionsBuilder
    where
        N: Into<String>,
        V: Serialize,
    {
        self.fields.insert(
            name.into(),
            serde_json::to_value(value).expect("Value to serialize"),
        );
        self
    }

    /// Updates resolution in transition
    pub fn resolution<R>(&mut self, name: R) -> &mut TransitionTriggerOptionsBuilder
    where
        R: Into<String>,
    {
        self.field("resolution", Resolution { name: name.into() });
        self
    }

    /// Adds a comment to the transition
    ///
    /// Automatically converts plain text to ADF format for v3 API compatibility
    pub fn comment<C>(&mut self, comment: C) -> &mut TransitionTriggerOptionsBuilder
    where
        C: Into<String>,
    {
        // Convert plain text to ADF format
        let adf = AdfDocument::from_text(comment.into());
        self.field("comment", serde_json::json!({"body": adf}));
        self
    }

    pub fn build(&self) -> TransitionTriggerOptions {
        TransitionTriggerOptions {
            transition: self.transition.clone(),
            fields: self.fields.clone(),
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Resolution {
    name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Transition {
    pub id: String,
}

/// Represents a worklog entry on an issue
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Worklog {
    #[serde(rename = "self")]
    pub self_link: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<TextContent>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    pub created: Option<OffsetDateTime>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    pub updated: Option<OffsetDateTime>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    pub started: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
}

/// Request to create or update a worklog
///
/// **API Compatibility:**
/// - JIRA Cloud (v3): Requires comments in Atlassian Document Format (ADF)
/// - JIRA Server/Data Center (v2): Requires comments as plain strings
///
/// This type automatically adapts based on the detected JIRA deployment type.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorklogInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "jira_datetime")]
    pub started: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent: Option<String>,
}

impl WorklogInput {
    /// Convert to JSON payload for JIRA Cloud v3 API (with ADF comment format)
    pub(crate) fn to_v3_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        // Convert comment to ADF format (required for v3)
        let comment_text = self.comment.as_deref().unwrap_or("");
        let adf_comment = AdfDocument::from_text(comment_text);
        obj.insert(
            "comment".to_string(),
            serde_json::to_value(adf_comment).unwrap(),
        );

        if let Some(started) = self.started {
            let format = time::format_description::parse(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory][offset_minute]"
            ).unwrap();
            let formatted = started.format(&format).unwrap();
            obj.insert("started".to_string(), serde_json::Value::String(formatted));
        }

        if let Some(time_spent_seconds) = self.time_spent_seconds {
            obj.insert(
                "timeSpentSeconds".to_string(),
                serde_json::Value::Number(time_spent_seconds.into()),
            );
        }

        if let Some(ref time_spent) = self.time_spent {
            obj.insert(
                "timeSpent".to_string(),
                serde_json::Value::String(time_spent.clone()),
            );
        }

        serde_json::Value::Object(obj)
    }
}

impl WorklogInput {
    /// Create a new worklog with time spent in seconds
    pub fn new(time_spent_seconds: u64) -> Self {
        Self {
            comment: None,
            started: None,
            time_spent_seconds: Some(time_spent_seconds),
            time_spent: None,
        }
    }

    /// Create a new worklog with time spent in minutes
    pub fn from_minutes(minutes: u64) -> Self {
        Self::new(minutes * 60)
    }

    /// Create a new worklog with time spent in hours
    pub fn from_hours(hours: u64) -> Self {
        Self::new(hours * 3600)
    }

    /// Create a new worklog with time spent in days (8 hour workday)
    pub fn from_days(days: u64) -> Self {
        Self::new(days * 8 * 3600)
    }

    /// Create a new worklog with time spent in weeks (5 day workweek, 8 hour days)
    pub fn from_weeks(weeks: u64) -> Self {
        Self::new(weeks * 5 * 8 * 3600)
    }

    /// Set a comment for the worklog
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Get the comment text
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    /// Set when the work was started
    pub fn with_started(mut self, started: OffsetDateTime) -> Self {
        self.started = Some(started);
        self
    }

    /// Set started time to N hours ago from now
    pub fn started_hours_ago(mut self, hours: i64) -> Self {
        let started = OffsetDateTime::now_utc() - time::Duration::hours(hours);
        self.started = Some(started);
        self
    }

    /// Set started time to N minutes ago from now
    pub fn started_minutes_ago(mut self, minutes: i64) -> Self {
        let started = OffsetDateTime::now_utc() - time::Duration::minutes(minutes);
        self.started = Some(started);
        self
    }

    /// Set started time to N days ago from now
    pub fn started_days_ago(mut self, days: i64) -> Self {
        let started = OffsetDateTime::now_utc() - time::Duration::days(days);
        self.started = Some(started);
        self
    }

    /// Set started time to N weeks ago from now
    pub fn started_weeks_ago(mut self, weeks: i64) -> Self {
        let started = OffsetDateTime::now_utc() - time::Duration::weeks(weeks);
        self.started = Some(started);
        self
    }

    /// Set started time to a specific date and time
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gouqi::WorklogInput;
    /// use time::macros::datetime;
    ///
    /// let worklog = WorklogInput::from_hours(2)
    ///     .started_at(datetime!(2024-01-15 09:00:00 UTC));
    /// ```
    pub fn started_at(mut self, datetime: OffsetDateTime) -> Self {
        self.started = Some(datetime);
        self
    }

    /// Set time spent as a string (e.g., "3h 20m")
    pub fn with_time_spent(mut self, time_spent: impl Into<String>) -> Self {
        self.time_spent = Some(time_spent.into());
        self
    }
}

/// Response containing a list of worklogs
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorklogList {
    pub start_at: u64,
    pub max_results: u64,
    pub total: u64,
    pub worklogs: Vec<Worklog>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub name: String,
}
