//! Interfaces for accessing and managing components

use serde::{Deserialize, Serialize};

// Ours
use crate::{Component, Jira, Result};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateComponent {
    pub name: String,
    pub description: Option<String>,
    pub project: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateComponentResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub project: String,
    #[serde(rename = "self")]
    pub url: String,
}

#[derive(Debug)]
pub struct Components {
    jira: Jira,
}

impl Components {
    pub fn new(jira: &Jira) -> Components {
        Components { jira: jira.clone() }
    }

    /// Get a single component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-getComponent)
    /// for more information
    pub fn get<I>(&self, id: I) -> Result<Component>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/component/{}", id.into()))
    }

    /// Create a new component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-createComponent)
    /// for more information
    pub fn create(&self, data: CreateComponent) -> Result<CreateComponentResponse> {
        self.jira.post("api", "/component", data)
    }

    /// Update a component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-updateComponent)
    /// for more information
    pub fn update<I>(&self, id: I, data: CreateComponent) -> Result<CreateComponentResponse>
    where
        I: Into<String>,
    {
        self.jira
            .put("api", &format!("/component/{}", id.into()), data)
    }

    /// Edit a component
    ///
    /// # Deprecated
    ///
    /// Use [`Components::update`] instead. This method will be removed in a future version.
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-updateComponent)
    /// for more information
    #[deprecated(
        since = "0.16.0",
        note = "Use `update` instead for consistency with REST conventions"
    )]
    pub fn edit<I>(&self, id: I, data: CreateComponent) -> Result<CreateComponentResponse>
    where
        I: Into<String>,
    {
        self.update(id, data)
    }

    /// Returns all components of a project
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/project-getProjectComponents)
    /// for more information
    pub fn list<I>(&self, project_id_or_key: I) -> Result<Vec<Component>>
    where
        I: Into<String>,
    {
        let path = format!("/project/{}/components", project_id_or_key.into());

        self.jira.get::<Vec<Component>>("api", &path)
    }

    /// Delete a component
    ///
    /// See this [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-components/#api-rest-api-2-component-id-delete)
    /// for more information
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/component/{}", id.into()))?;
        Ok(())
    }
}

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncComponents {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncComponents {
    pub fn new(jira: &AsyncJira) -> AsyncComponents {
        AsyncComponents { jira: jira.clone() }
    }

    /// Get a single component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-getComponent)
    /// for more information
    pub async fn get<I>(&self, id: I) -> Result<Component>
    where
        I: Into<String>,
    {
        self.jira
            .get("api", &format!("/component/{}", id.into()))
            .await
    }

    /// Create a new component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-createComponent)
    /// for more information
    pub async fn create(&self, data: CreateComponent) -> Result<CreateComponentResponse> {
        self.jira.post("api", "/component", data).await
    }

    /// Update a component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-updateComponent)
    /// for more information
    pub async fn update<I>(&self, id: I, data: CreateComponent) -> Result<CreateComponentResponse>
    where
        I: Into<String>,
    {
        self.jira
            .put("api", &format!("/component/{}", id.into()), data)
            .await
    }

    /// Edit a component
    ///
    /// # Deprecated
    ///
    /// Use [`AsyncComponents::update`] instead. This method will be removed in a future version.
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-updateComponent)
    /// for more information
    #[deprecated(
        since = "0.16.0",
        note = "Use `update` instead for consistency with REST conventions"
    )]
    pub async fn edit<I>(&self, id: I, data: CreateComponent) -> Result<CreateComponentResponse>
    where
        I: Into<String>,
    {
        self.update(id, data).await
    }

    /// Returns all components of a project
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/project-getProjectComponents)
    /// for more information
    pub async fn list<I>(&self, project_id_or_key: I) -> Result<Vec<Component>>
    where
        I: Into<String>,
    {
        let path = format!("/project/{}/components", project_id_or_key.into());
        self.jira.get::<Vec<Component>>("api", &path).await
    }

    /// Delete a component
    ///
    /// See this [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-components/#api-rest-api-2-component-id-delete)
    /// for more information
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/component/{}", id.into()))
            .await?;
        Ok(())
    }
}
