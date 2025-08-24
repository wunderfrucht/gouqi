//! MCP (Model Context Protocol) utilities for integrating Jira entities as MCP resources.
//!
//! This module provides utilities to convert Jira entities into MCP-compatible resources,
//! generate tool schemas, and handle MCP-specific operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::{Error, Issue, Project, User, Version, Board, Sprint, ProjectComponent};

/// URI schemes for different Jira resources
pub const JIRA_ISSUE_SCHEME: &str = "jira://issue/";
pub const JIRA_PROJECT_SCHEME: &str = "jira://project/";
pub const JIRA_USER_SCHEME: &str = "jira://user/";
pub const JIRA_COMPONENT_SCHEME: &str = "jira://component/";
pub const JIRA_VERSION_SCHEME: &str = "jira://version/";
pub const JIRA_BOARD_SCHEME: &str = "jira://board/";
pub const JIRA_SPRINT_SCHEME: &str = "jira://sprint/";

/// MCP Resource representation for Jira entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResource {
    /// Unique URI identifying the resource
    pub uri: String,
    /// Human-readable name of the resource
    pub name: String,
    /// Optional description of the resource
    pub description: Option<String>,
    /// MIME type of the resource content
    pub mime_type: String,
    /// Optional annotations for additional metadata
    pub annotations: Option<HashMap<String, serde_json::Value>>,
}

/// MCP Tool schema for Jira operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
}

/// MCP-compatible error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional error data
    pub data: Option<serde_json::Value>,
}

/// Trait for converting Jira entities to MCP resources
pub trait ToMCPResource {
    /// Convert the entity to an MCP resource
    fn to_mcp_resource(&self, base_uri: &str) -> MCPResource;
}

/// Resource URI management utilities
pub mod uri {
    use super::*;

    /// Generate a Jira issue URI
    pub fn issue_uri(key: &str) -> String {
        format!("{}{}", JIRA_ISSUE_SCHEME, key)
    }

    /// Generate a Jira project URI
    pub fn project_uri(key: &str) -> String {
        format!("{}{}", JIRA_PROJECT_SCHEME, key)
    }

    /// Generate a Jira user URI
    pub fn user_uri(account_id: &str) -> String {
        format!("{}{}", JIRA_USER_SCHEME, account_id)
    }

    /// Generate a Jira component URI
    pub fn component_uri(id: &str) -> String {
        format!("{}{}", JIRA_COMPONENT_SCHEME, id)
    }

    /// Generate a Jira version URI
    pub fn version_uri(id: &str) -> String {
        format!("{}{}", JIRA_VERSION_SCHEME, id)
    }

    /// Generate a Jira board URI
    pub fn board_uri(id: &str) -> String {
        format!("{}{}", JIRA_BOARD_SCHEME, id)
    }

    /// Generate a Jira sprint URI
    pub fn sprint_uri(id: &str) -> String {
        format!("{}{}", JIRA_SPRINT_SCHEME, id)
    }

    /// Parse a Jira resource URI and extract the resource type and identifier
    pub fn parse_jira_uri(uri: &str) -> Result<(String, String), Error> {
        let url = Url::parse(uri).map_err(Error::ParseError)?;

        let scheme = url.scheme();
        if scheme != "jira" {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        let host = url.host_str().unwrap_or("");
        let path = url.path().trim_start_matches('/');

        Ok((host.to_string(), path.to_string()))
    }

    /// Validate a Jira resource URI
    pub fn validate_jira_uri(uri: &str) -> Result<(), Error> {
        let url = Url::parse(uri).map_err(Error::ParseError)?;

        if url.scheme() != "jira" {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        let host = url.host_str().unwrap_or("");
        match host {
            "issue" | "project" | "user" | "component" | "version" => Ok(()),
            _ => Err(Error::ParseError(url::ParseError::EmptyHost)),
        }
    }
}

/// Tool schema generation utilities
pub mod schema {
    use super::*;

    /// Generate MCP tool schema for issue search
    pub fn issue_search_tool() -> MCPTool {
        MCPTool {
            name: "jira_search_issues".to_string(),
            description: "Search for Jira issues using JQL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "jql": {
                        "type": "string",
                        "description": "JQL (Jira Query Language) string"
                    },
                    "start_at": {
                        "type": "integer",
                        "description": "Starting index for pagination",
                        "minimum": 0,
                        "default": 0
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 50
                    },
                    "fields": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Fields to include in the response"
                    }
                },
                "required": ["jql"]
            }),
        }
    }

    /// Generate MCP tool schema for getting an issue
    pub fn get_issue_tool() -> MCPTool {
        MCPTool {
            name: "jira_get_issue".to_string(),
            description: "Get a specific Jira issue by key".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "issue_key": {
                        "type": "string",
                        "description": "Jira issue key (e.g., 'DEMO-123')",
                        "pattern": "^[A-Z]+-[0-9]+$"
                    },
                    "fields": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Fields to include in the response"
                    },
                    "expand": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Additional data to expand in the response"
                    }
                },
                "required": ["issue_key"]
            }),
        }
    }

    /// Generate MCP tool schema for creating an issue
    pub fn create_issue_tool() -> MCPTool {
        MCPTool {
            name: "jira_create_issue".to_string(),
            description: "Create a new Jira issue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project key or ID"
                    },
                    "issue_type": {
                        "type": "string",
                        "description": "Issue type (e.g., 'Bug', 'Task', 'Story')"
                    },
                    "summary": {
                        "type": "string",
                        "description": "Issue summary/title",
                        "maxLength": 255
                    },
                    "description": {
                        "type": "string",
                        "description": "Issue description"
                    },
                    "assignee": {
                        "type": "string",
                        "description": "Account ID of the assignee"
                    },
                    "priority": {
                        "type": "string",
                        "description": "Issue priority"
                    },
                    "labels": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Issue labels"
                    },
                    "components": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Component IDs or names"
                    }
                },
                "required": ["project", "issue_type", "summary"]
            }),
        }
    }

    /// Generate MCP tool schema for listing projects
    pub fn list_projects_tool() -> MCPTool {
        MCPTool {
            name: "jira_list_projects".to_string(),
            description: "List all available Jira projects".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "recent": {
                        "type": "integer",
                        "description": "Number of recently accessed projects to return",
                        "minimum": 1,
                        "maximum": 20
                    },
                    "expand": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Additional data to expand in the response"
                    }
                }
            }),
        }
    }

    /// Generate MCP tool schema for listing issue transitions
    pub fn list_issue_transitions_tool() -> MCPTool {
        MCPTool {
            name: "jira_list_issue_transitions".to_string(),
            description: "List available transitions for a specific issue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "issue_key": {
                        "type": "string",
                        "description": "Jira issue key (e.g., 'DEMO-123')",
                        "pattern": "^[A-Z]+-[0-9]+$"
                    }
                },
                "required": ["issue_key"]
            }),
        }
    }

    /// Generate MCP tool schema for triggering issue transitions
    pub fn trigger_issue_transition_tool() -> MCPTool {
        MCPTool {
            name: "jira_trigger_issue_transition".to_string(),
            description: "Trigger a transition on a specific issue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "issue_key": {
                        "type": "string",
                        "description": "Jira issue key (e.g., 'DEMO-123')",
                        "pattern": "^[A-Z]+-[0-9]+$"
                    },
                    "transition_id": {
                        "type": "string",
                        "description": "ID of the transition to trigger"
                    },
                    "comment": {
                        "type": "string",
                        "description": "Optional comment for the transition"
                    },
                    "resolution": {
                        "type": "string",
                        "description": "Optional resolution when transitioning to resolved states"
                    },
                    "fields": {
                        "type": "object",
                        "description": "Additional fields to update during transition",
                        "additionalProperties": true
                    }
                },
                "required": ["issue_key", "transition_id"]
            }),
        }
    }

    /// Generate MCP tool schema for listing issue attachments
    pub fn list_issue_attachments_tool() -> MCPTool {
        MCPTool {
            name: "jira_list_issue_attachments".to_string(),
            description: "List all attachments for a specific issue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "issue_key": {
                        "type": "string",
                        "description": "Jira issue key (e.g., 'DEMO-123')",
                        "pattern": "^[A-Z]+-[0-9]+$"
                    }
                },
                "required": ["issue_key"]
            }),
        }
    }

    /// Generate MCP tool schema for uploading issue attachments
    pub fn upload_issue_attachment_tool() -> MCPTool {
        MCPTool {
            name: "jira_upload_issue_attachment".to_string(),
            description: "Upload an attachment to a specific issue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "issue_key": {
                        "type": "string",
                        "description": "Jira issue key (e.g., 'DEMO-123')",
                        "pattern": "^[A-Z]+-[0-9]+$"
                    },
                    "filename": {
                        "type": "string",
                        "description": "Name of the file to upload"
                    },
                    "content": {
                        "type": "string",
                        "description": "Base64-encoded file content or file path"
                    },
                    "content_type": {
                        "type": "string",
                        "description": "MIME type of the file (e.g., 'image/png', 'text/plain')"
                    }
                },
                "required": ["issue_key", "filename", "content"]
            }),
        }
    }

    /// Generate MCP tool schema for creating project components
    pub fn create_project_component_tool() -> MCPTool {
        MCPTool {
            name: "jira_create_project_component".to_string(),
            description: "Create a new component in a project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project key or ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "Component name",
                        "maxLength": 255
                    },
                    "description": {
                        "type": "string",
                        "description": "Component description"
                    },
                    "lead": {
                        "type": "string",
                        "description": "Account ID of the component lead"
                    }
                },
                "required": ["project", "name"]
            }),
        }
    }

    /// Generate MCP tool schema for updating project components
    pub fn update_project_component_tool() -> MCPTool {
        MCPTool {
            name: "jira_update_project_component".to_string(),
            description: "Update an existing project component".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "component_id": {
                        "type": "string",
                        "description": "Component ID to update"
                    },
                    "name": {
                        "type": "string",
                        "description": "Updated component name",
                        "maxLength": 255
                    },
                    "description": {
                        "type": "string",
                        "description": "Updated component description"
                    },
                    "lead": {
                        "type": "string",
                        "description": "Account ID of the component lead"
                    }
                },
                "required": ["component_id"]
            }),
        }
    }

    /// Get all available MCP tools for Jira operations
    pub fn all_tools() -> Vec<MCPTool> {
        vec![
            // Core CRUD operations
            issue_search_tool(),
            get_issue_tool(),
            create_issue_tool(),
            list_projects_tool(),
            // Transition operations
            list_issue_transitions_tool(),
            trigger_issue_transition_tool(),
            // Attachment operations
            list_issue_attachments_tool(),
            upload_issue_attachment_tool(),
            // Component operations
            create_project_component_tool(),
            update_project_component_tool(),
        ]
    }
}

/// Error mapping utilities
pub mod error {
    use super::*;
    use reqwest::StatusCode;

    /// Convert Jira API errors to MCP-compatible format
    pub fn to_mcp_error(error: &Error) -> MCPError {
        match error {
            Error::Http(reqwest_error) => {
                let status = reqwest_error
                    .status()
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                MCPError {
                    code: status.as_u16() as i32,
                    message: format!("HTTP error: {}", reqwest_error),
                    data: Some(serde_json::json!({
                        "type": "http_error",
                        "status_code": status.as_u16()
                    })),
                }
            }
            Error::Unauthorized => MCPError {
                code: 401,
                message: "Unauthorized access to Jira".to_string(),
                data: Some(serde_json::json!({
                    "type": "authentication_error"
                })),
            },
            Error::NotFound => MCPError {
                code: 404,
                message: "Resource not found".to_string(),
                data: Some(serde_json::json!({
                    "type": "not_found_error"
                })),
            },
            Error::MethodNotAllowed => MCPError {
                code: 405,
                message: "Method not allowed".to_string(),
                data: Some(serde_json::json!({
                    "type": "method_not_allowed_error"
                })),
            },
            Error::Fault { code, errors } => MCPError {
                code: code.as_u16() as i32,
                message: format!("Jira API error: {}", code),
                data: Some(serde_json::json!({
                    "type": "jira_api_error",
                    "status_code": code.as_u16(),
                    "jira_errors": errors
                })),
            },
            Error::Serde(serde_error) => MCPError {
                code: 422,
                message: format!("Data serialization error: {}", serde_error),
                data: Some(serde_json::json!({
                    "type": "serialization_error"
                })),
            },
            Error::IO(io_error) => MCPError {
                code: 500,
                message: format!("I/O error: {}", io_error),
                data: Some(serde_json::json!({
                    "type": "io_error"
                })),
            },
            Error::ParseError(parse_error) => MCPError {
                code: 400,
                message: format!("Parse error: {}", parse_error),
                data: Some(serde_json::json!({
                    "type": "parse_error"
                })),
            },
        }
    }
}

/// Input validation utilities
pub mod validation {
    use super::*;

    /// Validate issue key format
    pub fn validate_issue_key(key: &str) -> Result<(), Error> {
        if key.is_empty() {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        // Issue key should match pattern: PROJECT-123
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() != 2 {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        let project_key = parts[0];
        let issue_number = parts[1];

        // Project key should be uppercase letters and not empty
        if project_key.is_empty() || !project_key.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        // Issue number should be numeric and not empty
        if issue_number.is_empty() || !issue_number.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        Ok(())
    }

    /// Validate project key format
    pub fn validate_project_key(key: &str) -> Result<(), Error> {
        if key.is_empty() || key.len() > 10 {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        // Project key should be uppercase letters and numbers, but digits don't have uppercase
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
        {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        Ok(())
    }

    /// Validate JQL query for basic safety
    pub fn validate_jql(jql: &str) -> Result<(), Error> {
        if jql.is_empty() {
            return Err(Error::ParseError(url::ParseError::EmptyHost));
        }

        // Basic validation - ensure no obviously dangerous patterns
        let dangerous_patterns = [
            "DROP", "DELETE", "INSERT", "UPDATE", "UNION", "--", "/*", "*/",
        ];
        let upper_jql = jql.to_uppercase();

        for pattern in &dangerous_patterns {
            if upper_jql.contains(pattern) {
                return Err(Error::ParseError(url::ParseError::EmptyHost));
            }
        }

        Ok(())
    }

    /// Validate pagination parameters
    pub fn validate_pagination(
        start_at: Option<i32>,
        max_results: Option<i32>,
    ) -> Result<(), Error> {
        if let Some(start) = start_at {
            if start < 0 {
                return Err(Error::ParseError(url::ParseError::EmptyHost));
            }
        }

        if let Some(max) = max_results {
            if !(1..=1000).contains(&max) {
                return Err(Error::ParseError(url::ParseError::EmptyHost));
            }
        }

        Ok(())
    }
}

/// Implementation of ToMCPResource for Issue
impl ToMCPResource for Issue {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();

        // Use the Issue helper methods to access typed fields
        if let Some(project) = self.project() {
            annotations.insert("project".to_string(), serde_json::json!(project.key));
        }

        if let Some(status) = self.status() {
            annotations.insert("status".to_string(), serde_json::json!(status.name));
        }

        if let Some(issue_type) = self.issue_type() {
            annotations.insert("issue_type".to_string(), serde_json::json!(issue_type.name));
        }

        if let Some(assignee) = self.assignee() {
            annotations.insert(
                "assignee".to_string(),
                serde_json::json!(assignee.display_name),
            );
        }

        let summary = self.summary().unwrap_or_else(|| "No summary".to_string());

        MCPResource {
            uri: uri::issue_uri(&self.key),
            name: format!("{}: {}", self.key, summary),
            description: self.description(),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for Project
impl ToMCPResource for Project {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert(
            "project_type".to_string(),
            serde_json::json!(self.project_type_key),
        );

        if let Some(lead) = &self.lead {
            annotations.insert("lead".to_string(), serde_json::json!(lead.display_name));
        }

        MCPResource {
            uri: uri::project_uri(&self.key),
            name: format!("{}: {}", self.key, self.name),
            description: self.description.clone(),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for User
impl ToMCPResource for User {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert("active".to_string(), serde_json::json!(self.active));
        
        if let Some(email) = &self.email_address {
            annotations.insert("email".to_string(), serde_json::json!(email));
        }
        
        if let Some(key) = &self.key {
            annotations.insert("key".to_string(), serde_json::json!(key));
        }
        
        if let Some(name) = &self.name {
            annotations.insert("username".to_string(), serde_json::json!(name));
        }
        
        if let Some(timezone) = &self.timezone {
            annotations.insert("timezone".to_string(), serde_json::json!(timezone));
        }

        MCPResource {
            uri: uri::user_uri(&self.display_name),
            name: format!("User: {}", self.display_name),
            description: Some(format!("Jira user {}", self.display_name)),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for Version
impl ToMCPResource for Version {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert("project_id".to_string(), serde_json::json!(self.project_id));
        annotations.insert("released".to_string(), serde_json::json!(self.released));
        annotations.insert("archived".to_string(), serde_json::json!(self.archived));
        
        let status = if self.archived {
            "Archived"
        } else if self.released {
            "Released"
        } else {
            "Unreleased"
        };
        annotations.insert("status".to_string(), serde_json::json!(status));

        MCPResource {
            uri: uri::version_uri(&self.id),
            name: format!("Version: {}", self.name),
            description: Some(format!("Project version {} ({})", self.name, status)),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for Board
impl ToMCPResource for Board {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert("board_type".to_string(), serde_json::json!(self.type_name));
        annotations.insert("board_id".to_string(), serde_json::json!(self.id));
        
        if let Some(location) = &self.location {
            if let Some(project_id) = location.project_id {
                annotations.insert("project_id".to_string(), serde_json::json!(project_id));
            }
            if let Some(user_id) = location.user_id {
                annotations.insert("user_id".to_string(), serde_json::json!(user_id));
            }
        }

        MCPResource {
            uri: format!("jira://board/{}", self.id),
            name: format!("Board: {}", self.name),
            description: Some(format!("Jira {} board: {}", self.type_name, self.name)),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for Sprint
impl ToMCPResource for Sprint {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert("sprint_id".to_string(), serde_json::json!(self.id));
        
        if let Some(state) = &self.state {
            annotations.insert("state".to_string(), serde_json::json!(state));
        }
        
        if let Some(start_date) = &self.start_date {
            annotations.insert("start_date".to_string(), serde_json::json!(start_date.to_string()));
        }
        
        if let Some(end_date) = &self.end_date {
            annotations.insert("end_date".to_string(), serde_json::json!(end_date.to_string()));
        }
        
        if let Some(complete_date) = &self.complete_date {
            annotations.insert("complete_date".to_string(), serde_json::json!(complete_date.to_string()));
        }
        
        if let Some(board_id) = &self.origin_board_id {
            annotations.insert("origin_board_id".to_string(), serde_json::json!(board_id));
        }

        let description = format!("Sprint: {} ({})", self.name, 
            self.state.as_ref().unwrap_or(&"UNKNOWN".to_string()));

        MCPResource {
            uri: format!("jira://sprint/{}", self.id),
            name: format!("Sprint: {}", self.name),
            description: Some(description),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}

/// Implementation of ToMCPResource for ProjectComponent
impl ToMCPResource for ProjectComponent {
    fn to_mcp_resource(&self, _base_uri: &str) -> MCPResource {
        let mut annotations = HashMap::new();
        annotations.insert("component_id".to_string(), serde_json::json!(self.id));

        MCPResource {
            uri: uri::component_uri(&self.id),
            name: format!("Component: {}", self.name),
            description: self.description.clone()
                .or_else(|| Some(format!("Project component: {}", self.name))),
            mime_type: "application/json".to_string(),
            annotations: Some(annotations),
        }
    }
}
