//! Interfaces for accessing and managing issues

// Third party
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::form_urlencoded;

// Ours
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

#[derive(Debug, Serialize)]
pub struct AddComment {
    pub body: String,
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

    /// Edit an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn edit<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira.put("api", &format!("/issue/{}", id.into()), data)
    }

    /// Edit an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub fn edit_custom_issue<I, T>(&self, id: I, data: EditCustomIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira.put("api", &format!("/issue/{}", id.into()), data)
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

    pub fn comment<K>(&self, key: K, data: AddComment) -> Result<Comment>
    where
        K: Into<String>,
    {
        self.jira.post(
            "api",
            format!("/issue/{}/comment", key.into()).as_ref(),
            data,
        )
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
        self.jira.delete("api", &format!("/issue/{}", id.into()))
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
                    _ => None,
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

    /// Create a new issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-createIssue)
    /// for more information
    pub async fn create(&self, data: CreateIssue) -> Result<CreateResponse> {
        self.jira.post("api", "/issue", data).await
    }

    /// Edit an issue
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/issue-editIssue)
    /// for more information
    pub async fn edit<I, T>(&self, id: I, data: EditIssue<T>) -> Result<()>
    where
        I: Into<String>,
        T: Serialize,
    {
        self.jira
            .put("api", &format!("/issue/{}", id.into()), data)
            .await
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

    pub async fn comment<K>(&self, key: K, data: AddComment) -> Result<Comment>
    where
        K: Into<String>,
    {
        self.jira
            .post(
                "api",
                format!("/issue/{}/comment", key.into()).as_ref(),
                data,
            )
            .await
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
            .delete("api", &format!("/issue/{}", id.into()))
            .await
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
