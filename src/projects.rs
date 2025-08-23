//! Interfaces for accessing and managing projects

use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::{
    CreateProject, Jira, Project, ProjectComponent, ProjectRole, ProjectSearchOptions,
    ProjectSearchResults, Result, UpdateProject, Version,
};

/// Project interface
#[derive(Debug)]
pub struct Projects {
    jira: Jira,
}

impl Projects {
    pub fn new(jira: &Jira) -> Projects {
        Projects { jira: jira.clone() }
    }

    /// List all projects accessible to the user
    pub fn list(&self) -> Result<Vec<Project>> {
        self.jira.get::<Vec<Project>>("api", "/project")
    }

    /// Get a specific project by ID or key
    /// # Panics
    /// This function will panic if the project ID is invalid
    pub fn get<I>(&self, id: I) -> Result<Project>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira.get::<Project>("api", &format!("/project/{}", id))
    }

    /// Create a new project
    /// # Panics
    /// This function will panic if project creation fails due to permissions or validation
    pub fn create(&self, project: CreateProject) -> Result<Project> {
        self.jira
            .post::<Project, CreateProject>("api", "/project", project)
    }

    /// Update an existing project
    /// # Panics
    /// This function will panic if the project update fails
    pub fn update<I>(&self, id: I, project: UpdateProject) -> Result<Project>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira
            .put::<Project, UpdateProject>("api", &format!("/project/{}", id), project)
    }

    /// Delete a project (requires admin permissions)
    /// # Panics
    /// This function will panic if project deletion fails
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/project/{}", id))?;
        Ok(())
    }

    /// Get all versions for a project
    pub fn get_versions<I>(&self, project_id: I) -> Result<Vec<Version>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<Vec<Version>>("api", &format!("/project/{}/versions", project_id))
    }

    /// Get all components for a project
    pub fn get_components<I>(&self, project_id: I) -> Result<Vec<ProjectComponent>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<Vec<ProjectComponent>>("api", &format!("/project/{}/components", project_id))
    }

    /// Search projects with query parameters
    pub fn search(&self, options: &ProjectSearchOptions) -> Result<ProjectSearchResults> {
        let mut path = vec!["/project/search".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        if !query_options.is_empty() {
            let query = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&query_options)
                .finish();
            path.push(query);
        }
        self.jira
            .get::<ProjectSearchResults>("api", &path.join("?"))
    }

    /// Get project roles
    pub fn get_roles<I>(&self, project_id: I) -> Result<BTreeMap<String, String>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<BTreeMap<String, String>>("api", &format!("/project/{}/role", project_id))
    }

    /// Get users for a specific project role
    pub fn get_role_users<I>(&self, project_id: I, role_id: u64) -> Result<ProjectRole>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<ProjectRole>("api", &format!("/project/{}/role/{}", project_id, role_id))
    }
}

// Async implementation when async feature is enabled
#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
/// Async Project interface
#[derive(Debug)]
pub struct AsyncProjects {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncProjects {
    pub fn new(jira: &AsyncJira) -> AsyncProjects {
        AsyncProjects { jira: jira.clone() }
    }

    /// List all projects accessible to the user
    pub async fn list(&self) -> Result<Vec<Project>> {
        self.jira.get::<Vec<Project>>("api", "/project").await
    }

    /// Get a specific project by ID or key
    /// # Panics
    /// This function will panic if the project ID is invalid
    pub async fn get<I>(&self, id: I) -> Result<Project>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira
            .get::<Project>("api", &format!("/project/{}", id))
            .await
    }

    /// Create a new project
    /// # Panics
    /// This function will panic if project creation fails due to permissions or validation
    pub async fn create(&self, project: CreateProject) -> Result<Project> {
        self.jira
            .post::<Project, CreateProject>("api", "/project", project)
            .await
    }

    /// Update an existing project
    /// # Panics
    /// This function will panic if the project update fails
    pub async fn update<I>(&self, id: I, project: UpdateProject) -> Result<Project>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira
            .put::<Project, UpdateProject>("api", &format!("/project/{}", id), project)
            .await
    }

    /// Delete a project (requires admin permissions)
    /// # Panics
    /// This function will panic if project deletion fails
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        let id = id.into();
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/project/{}", id))
            .await?;
        Ok(())
    }

    /// Get all versions for a project
    pub async fn get_versions<I>(&self, project_id: I) -> Result<Vec<Version>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<Vec<Version>>("api", &format!("/project/{}/versions", project_id))
            .await
    }

    /// Get all components for a project
    pub async fn get_components<I>(&self, project_id: I) -> Result<Vec<ProjectComponent>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<Vec<ProjectComponent>>("api", &format!("/project/{}/components", project_id))
            .await
    }

    /// Search projects with query parameters
    pub async fn search(&self, options: &ProjectSearchOptions) -> Result<ProjectSearchResults> {
        let mut path = vec!["/project/search".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        if !query_options.is_empty() {
            let query = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&query_options)
                .finish();
            path.push(query);
        }
        self.jira
            .get::<ProjectSearchResults>("api", &path.join("?"))
            .await
    }

    /// Get project roles
    pub async fn get_roles<I>(&self, project_id: I) -> Result<BTreeMap<String, String>>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<BTreeMap<String, String>>("api", &format!("/project/{}/role", project_id))
            .await
    }

    /// Get users for a specific project role
    pub async fn get_role_users<I>(&self, project_id: I, role_id: u64) -> Result<ProjectRole>
    where
        I: Into<String>,
    {
        let project_id = project_id.into();
        self.jira
            .get::<ProjectRole>("api", &format!("/project/{}/role/{}", project_id, role_id))
            .await
    }
}
