//! Issue relationship management and graph structures
//!
//! This module provides functionality for managing Jira issue relationships
//! in a declarative, AI-friendly format. It supports traversing relationship
//! graphs to specified depths and applying relationship changes.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use time::OffsetDateTime;

// No additional imports needed - Result not used in this module

/// A declarative representation of issue relationships
///
/// This structure can represent both current state (extracted from Jira)
/// and desired state (to be applied to Jira). It's designed to be
/// AI-friendly with clear semantics and JSON/YAML compatibility.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationshipGraph {
    /// Map of issue key to its relationships
    pub issues: HashMap<String, IssueRelationships>,
    /// Metadata about the graph
    pub metadata: GraphMetadata,
}

/// Relationships for a single issue
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IssueRelationships {
    // Standard Jira relationship types
    /// Issues that this issue blocks
    pub blocks: Vec<String>,
    /// Issues that block this issue
    pub blocked_by: Vec<String>,
    /// Issues related to this issue
    pub relates_to: Vec<String>,
    /// Issues that this issue duplicates
    pub duplicates: Vec<String>,
    /// Issues that duplicate this issue
    pub duplicated_by: Vec<String>,

    // Hierarchical relationships
    /// Parent issue (subtask -> task relationship)
    pub parent: Option<String>,
    /// Child issues (task -> subtask relationship)
    pub children: Vec<String>,

    // Epic/Story relationships
    /// Epic that contains this issue
    pub epic: Option<String>,
    /// Stories contained in this epic
    pub stories: Vec<String>,

    // Custom relationship types
    /// Custom link types with their related issues
    pub custom: HashMap<String, Vec<String>>,
}

/// Metadata about the relationship graph
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphMetadata {
    /// Root issue used for traversal (if any)
    pub root_issue: Option<String>,
    /// Maximum depth traversed
    pub max_depth: u32,
    /// When this graph was created
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
    /// Source of the data
    pub source: String,
    /// Total number of issues in the graph
    pub issue_count: usize,
    /// Total number of relationships in the graph
    pub relationship_count: usize,
}

/// Options for relationship graph extraction
#[derive(Debug, Clone)]
pub struct GraphOptions {
    /// Include specific relationship types only
    pub include_types: Option<Vec<String>>,
    /// Exclude specific relationship types
    pub exclude_types: Option<Vec<String>>,
    /// Include custom link types
    pub include_custom: bool,
    /// Follow bidirectional relationships
    pub bidirectional: bool,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            include_types: None,
            exclude_types: None,
            include_custom: true,
            bidirectional: true,
        }
    }
}

/// Options for applying relationship graphs
#[derive(Debug, Clone, Default)]
pub struct ApplyOptions {
    /// Dry run - don't actually make changes
    pub dry_run: bool,
    /// Create missing issues if referenced
    pub create_missing_issues: bool,
    /// Maximum number of operations to perform
    pub max_operations: Option<usize>,
}

/// Result of applying a relationship graph
#[derive(Debug)]
pub struct ApplyResult {
    /// Links that were created
    pub created_links: Vec<CreatedLink>,
    /// Links that were deleted
    pub deleted_links: Vec<DeletedLink>,
    /// Errors that occurred during application
    pub errors: Vec<LinkError>,
    /// Summary of changes made
    pub summary: String,
}

/// Information about a created link
#[derive(Debug)]
pub struct CreatedLink {
    pub from_issue: String,
    pub to_issue: String,
    pub link_type: String,
    pub link_id: Option<String>,
}

/// Information about a deleted link
#[derive(Debug)]
pub struct DeletedLink {
    pub from_issue: String,
    pub to_issue: String,
    pub link_type: String,
    pub link_id: String,
}

/// Error that occurred during link operations
#[derive(Debug)]
pub struct LinkError {
    pub operation: String,
    pub from_issue: String,
    pub to_issue: String,
    pub link_type: String,
    pub error: String,
}

/// Difference between two relationship graphs
#[derive(Debug)]
pub struct RelationshipDiff {
    /// Links that need to be created
    pub links_to_create: Vec<LinkOperation>,
    /// Links that need to be deleted
    pub links_to_delete: Vec<LinkOperation>,
    /// Issues with unchanged relationships
    pub unchanged: Vec<String>,
}

/// A link operation (create or delete)
#[derive(Debug)]
pub struct LinkOperation {
    pub from_issue: String,
    pub to_issue: String,
    pub link_type: String,
}

impl RelationshipGraph {
    /// Create a new empty relationship graph
    pub fn new(source: String) -> Self {
        Self {
            issues: HashMap::new(),
            metadata: GraphMetadata {
                root_issue: None,
                max_depth: 0,
                timestamp: OffsetDateTime::now_utc(),
                source,
                issue_count: 0,
                relationship_count: 0,
            },
        }
    }

    /// Add an issue to the graph
    pub fn add_issue(&mut self, issue_key: String, relationships: IssueRelationships) {
        self.issues.insert(issue_key, relationships);
        self.update_metadata();
    }

    /// Get relationships for an issue
    pub fn get_relationships(&self, issue_key: &str) -> Option<&IssueRelationships> {
        self.issues.get(issue_key)
    }

    /// Get all issue keys in the graph
    pub fn get_issue_keys(&self) -> Vec<&String> {
        self.issues.keys().collect()
    }

    /// Check if the graph contains an issue
    pub fn contains_issue(&self, issue_key: &str) -> bool {
        self.issues.contains_key(issue_key)
    }

    /// Update metadata counters
    fn update_metadata(&mut self) {
        self.metadata.issue_count = self.issues.len();
        self.metadata.relationship_count = self.count_relationships();
    }

    /// Count total relationships in the graph
    fn count_relationships(&self) -> usize {
        self.issues
            .values()
            .map(|rel| {
                rel.blocks.len()
                    + rel.blocked_by.len()
                    + rel.relates_to.len()
                    + rel.duplicates.len()
                    + rel.duplicated_by.len()
                    + rel.children.len()
                    + if rel.parent.is_some() { 1 } else { 0 }
                    + if rel.epic.is_some() { 1 } else { 0 }
                    + rel.stories.len()
                    + rel.custom.values().map(|v| v.len()).sum::<usize>()
            })
            .sum()
    }

    /// Get all issues that are related to the given issue
    pub fn get_related_issues(&self, issue_key: &str) -> HashSet<String> {
        let mut related = HashSet::new();

        if let Some(relationships) = self.get_relationships(issue_key) {
            related.extend(relationships.blocks.iter().cloned());
            related.extend(relationships.blocked_by.iter().cloned());
            related.extend(relationships.relates_to.iter().cloned());
            related.extend(relationships.duplicates.iter().cloned());
            related.extend(relationships.duplicated_by.iter().cloned());
            related.extend(relationships.children.iter().cloned());
            related.extend(relationships.stories.iter().cloned());

            if let Some(parent) = &relationships.parent {
                related.insert(parent.clone());
            }
            if let Some(epic) = &relationships.epic {
                related.insert(epic.clone());
            }

            for custom_links in relationships.custom.values() {
                related.extend(custom_links.iter().cloned());
            }
        }

        related
    }

    /// Get the shortest path between two issues in the graph
    pub fn get_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            let related = self.get_related_issues(&current);

            for neighbor in related {
                if neighbor == to {
                    // Reconstruct path
                    let mut path = vec![to.to_string()];
                    let mut current_node = current;

                    while let Some(p) = parent.get(&current_node) {
                        path.push(current_node);
                        current_node = p.clone();
                    }
                    path.push(from.to_string());
                    path.reverse();
                    return Some(path);
                }

                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    parent.insert(neighbor.clone(), current.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        None
    }
}

impl IssueRelationships {
    /// Create new empty relationships
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this issue has any relationships
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
            && self.blocked_by.is_empty()
            && self.relates_to.is_empty()
            && self.duplicates.is_empty()
            && self.duplicated_by.is_empty()
            && self.children.is_empty()
            && self.parent.is_none()
            && self.epic.is_none()
            && self.stories.is_empty()
            && self.custom.is_empty()
    }

    /// Add a relationship of the given type
    pub fn add_relationship(&mut self, relationship_type: &str, target_issue: String) {
        match relationship_type {
            "blocks" => self.blocks.push(target_issue),
            "blocked_by" => self.blocked_by.push(target_issue),
            "relates_to" => self.relates_to.push(target_issue),
            "duplicates" => self.duplicates.push(target_issue),
            "duplicated_by" => self.duplicated_by.push(target_issue),
            "child" => self.children.push(target_issue),
            "parent" => self.parent = Some(target_issue),
            "epic" => self.epic = Some(target_issue),
            "story" => self.stories.push(target_issue),
            custom_type => {
                self.custom
                    .entry(custom_type.to_string())
                    .or_default()
                    .push(target_issue);
            }
        }
    }

    /// Remove a relationship of the given type
    pub fn remove_relationship(&mut self, relationship_type: &str, target_issue: &str) {
        match relationship_type {
            "blocks" => self.blocks.retain(|issue| issue != target_issue),
            "blocked_by" => self.blocked_by.retain(|issue| issue != target_issue),
            "relates_to" => self.relates_to.retain(|issue| issue != target_issue),
            "duplicates" => self.duplicates.retain(|issue| issue != target_issue),
            "duplicated_by" => self.duplicated_by.retain(|issue| issue != target_issue),
            "child" => self.children.retain(|issue| issue != target_issue),
            "parent" => {
                if self.parent.as_ref() == Some(&target_issue.to_string()) {
                    self.parent = None;
                }
            }
            "epic" => {
                if self.epic.as_ref() == Some(&target_issue.to_string()) {
                    self.epic = None;
                }
            }
            "story" => self.stories.retain(|issue| issue != target_issue),
            custom_type => {
                if let Some(custom_links) = self.custom.get_mut(custom_type) {
                    custom_links.retain(|issue| issue != target_issue);
                    if custom_links.is_empty() {
                        self.custom.remove(custom_type);
                    }
                }
            }
        }
    }

    /// Get all issues related through any relationship type
    pub fn get_all_related(&self) -> HashSet<String> {
        let mut related = HashSet::new();

        related.extend(self.blocks.iter().cloned());
        related.extend(self.blocked_by.iter().cloned());
        related.extend(self.relates_to.iter().cloned());
        related.extend(self.duplicates.iter().cloned());
        related.extend(self.duplicated_by.iter().cloned());
        related.extend(self.children.iter().cloned());
        related.extend(self.stories.iter().cloned());

        if let Some(parent) = &self.parent {
            related.insert(parent.clone());
        }
        if let Some(epic) = &self.epic {
            related.insert(epic.clone());
        }

        for custom_links in self.custom.values() {
            related.extend(custom_links.iter().cloned());
        }

        related
    }
}
