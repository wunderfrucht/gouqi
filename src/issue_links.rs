//! Issue Links API
//!
//! This module provides methods to manage links between Jira issues.

use crate::{CreateIssueLinkInput, IssueLink, Jira, Result};

/// Issue Links operations
pub struct IssueLinks {
    jira: Jira,
}

impl IssueLinks {
    pub(crate) fn new(jira: &Jira) -> Self {
        Self { jira: jira.clone() }
    }

    /// Get an issue link by ID
    ///
    /// Returns the details of a specific issue link.
    ///
    /// # Arguments
    ///
    /// * `link_id` - The ID of the issue link
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let link = jira.issue_links().get("10001")?;
    /// println!("Link type: {}", link.link_type.name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get<L>(&self, link_id: L) -> Result<IssueLink>
    where
        L: Into<String>,
    {
        self.jira
            .get("api", &format!("/issueLink/{}", link_id.into()))
    }

    /// Create a link between two issues
    ///
    /// Creates a new link between two issues with the specified link type.
    ///
    /// # Arguments
    ///
    /// * `link` - The link data including type, inward and outward issues
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira, CreateIssueLinkInput};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// let link = CreateIssueLinkInput::new("Blocks", "PROJ-1", "PROJ-2")
    ///     .with_comment("Blocking relationship");
    ///
    /// jira.issue_links().create(link)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create(&self, link: CreateIssueLinkInput) -> Result<()> {
        self.jira
            .post::<(), CreateIssueLinkInput>("api", "/issueLink", link)
    }

    /// Delete an issue link
    ///
    /// Removes the link between two issues.
    ///
    /// # Arguments
    ///
    /// * `link_id` - The ID of the issue link to delete
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gouqi::{Credentials, Jira};
    /// # let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    /// jira.issue_links().delete("10001")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete<L>(&self, link_id: L) -> Result<()>
    where
        L: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/issueLink/{}", link_id.into()))?;
        Ok(())
    }
}

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
/// Async issue links operations
pub struct AsyncIssueLinks {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncIssueLinks {
    pub(crate) fn new(jira: &AsyncJira) -> Self {
        Self { jira: jira.clone() }
    }

    /// Get an issue link by ID (async)
    pub async fn get<L>(&self, link_id: L) -> Result<IssueLink>
    where
        L: Into<String>,
    {
        self.jira
            .get("api", &format!("/issueLink/{}", link_id.into()))
            .await
    }

    /// Create a link between two issues (async)
    pub async fn create(&self, link: CreateIssueLinkInput) -> Result<()> {
        self.jira
            .post::<(), CreateIssueLinkInput>("api", "/issueLink", link)
            .await
    }

    /// Delete an issue link (async)
    pub async fn delete<L>(&self, link_id: L) -> Result<()>
    where
        L: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/issueLink/{}", link_id.into()))
            .await?;
        Ok(())
    }
}
