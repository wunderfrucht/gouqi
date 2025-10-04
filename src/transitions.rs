//! Interfaces for accessing and managing transition

// Ours
use crate::{Error, Jira, Result, TransitionOption, TransitionOptions, TransitionTriggerOptions};

/// Issue transition interface
#[derive(Debug)]
pub struct Transitions {
    jira: Jira,
    key: String,
}

impl Transitions {
    pub fn new<K>(jira: &Jira, key: K) -> Transitions
    where
        K: Into<String>,
    {
        Transitions {
            jira: jira.clone(),
            key: key.into(),
        }
    }

    /// Return list of transitions options for this issue
    pub fn list(&self) -> Result<Vec<TransitionOption>> {
        self.jira
            .get::<TransitionOptions>(
                "api",
                &format!("/issue/{}/transitions?expand=transitions.fields", self.key),
            )
            .map(|wrapper| wrapper.transitions)
    }

    /// Trigger a issue transition to transition with a resolution
    /// use TransitionTrigger::builder(id).resolution(name)
    pub fn trigger(&self, trans: TransitionTriggerOptions) -> Result<()> {
        self.jira
            .post::<(), TransitionTriggerOptions>(
                "api",
                &format!("/issue/{}/transitions", self.key),
                trans,
            )
            .or_else(|e| match e {
                Error::Serde(_) => Ok(()),
                e => Err(e),
            })
    }
}

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncTransitions {
    jira: AsyncJira,
    key: String,
}

#[cfg(feature = "async")]
impl AsyncTransitions {
    pub fn new<K>(jira: &AsyncJira, key: K) -> AsyncTransitions
    where
        K: Into<String>,
    {
        AsyncTransitions {
            jira: jira.clone(),
            key: key.into(),
        }
    }

    /// Return list of transitions options for this issue
    pub async fn list(&self) -> Result<Vec<TransitionOption>> {
        self.jira
            .get::<TransitionOptions>(
                "api",
                &format!("/issue/{}/transitions?expand=transitions.fields", self.key),
            )
            .await
            .map(|wrapper| wrapper.transitions)
    }

    /// Trigger a issue transition to transition with a resolution
    /// use TransitionTrigger::builder(id).resolution(name)
    pub async fn trigger(&self, trans: TransitionTriggerOptions) -> Result<()> {
        self.jira
            .post::<(), TransitionTriggerOptions>(
                "api",
                &format!("/issue/{}/transitions", self.key),
                trans,
            )
            .await
            .or_else(|e| match e {
                Error::Serde(_) => Ok(()),
                e => Err(e),
            })
    }
}
