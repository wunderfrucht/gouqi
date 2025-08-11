//! Interfaces for accessing and managing resolutions

// Third party
use serde::Deserialize;
use std::collections::BTreeMap;

// Ours
use crate::{Jira, Result};

#[derive(Debug)]
pub struct Resolution {
    jira: Jira,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Resolved {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub resolution_type: String,
    pub properties: BTreeMap<String, ::serde_json::Value>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: bool,
}

impl Resolution {
    pub fn new(jira: &Jira) -> Resolution {
        Resolution { jira: jira.clone() }
    }

    pub fn get<I>(&self, id: I) -> Result<Resolved>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/resolution/{}", id.into()))
    }
}
