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

    /// Edit a component
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/latest/#api/2/component-updateComponent)
    /// for more information
    pub fn edit<I>(&self, id: I, data: CreateComponent) -> Result<CreateComponentResponse>
    where
        I: Into<String>,
    {
        self.jira
            .put("api", &format!("/component/{}", id.into()), data)
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
}
