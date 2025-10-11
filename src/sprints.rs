//! Interfaces for accessing and managing sprints

// Third party
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::info;
use url::form_urlencoded;

// Ours
use crate::{Board, EmptyResponse, Jira, Result, SearchOptions};

#[derive(Debug)]
pub struct Sprints {
    jira: Jira,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Sprint {
    pub id: u64,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    pub state: Option<String>,
    #[serde(default, rename = "startDate", with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(default, rename = "endDate", with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(
        default,
        rename = "completeDate",
        with = "time::serde::iso8601::option"
    )]
    pub complete_date: Option<OffsetDateTime>,
    #[serde(rename = "originBoardId")]
    pub origin_board_id: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct CreateSprint {
    pub name: String,
    #[serde(rename = "originBoardId")]
    pub origin_board_id: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct UpdateSprint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "startDate",
        with = "crate::rep::jira_datetime"
    )]
    pub start_date: Option<OffsetDateTime>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "endDate",
        with = "crate::rep::jira_datetime"
    )]
    pub end_date: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>, // "future", "active", "closed"
}

#[derive(Deserialize, Debug)]
pub struct SprintResults {
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub values: Vec<Sprint>,
}

#[derive(Serialize, Debug)]
struct MoveIssues {
    issues: Vec<String>,
}

impl Sprints {
    pub fn new(jira: &Jira) -> Sprints {
        Sprints { jira: jira.clone() }
    }

    /// Create a new sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/9.5.0/#agile/1.0/sprint-createSprint)
    /// for more information
    ///     `pub fn create<T: Into<String>>(&self, project_id: u64, name: T) -> Result<Version> {`
    pub fn create<T: Into<String>>(&self, board: Board, name: T) -> Result<Sprint> {
        let data: CreateSprint = CreateSprint {
            name: name.into(),
            origin_board_id: Some(board.id),
        };
        info!("{:?}", data);
        self.jira.post("agile", "/sprint", data)
    }

    /// Get a single sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/sprint-getSprint)
    /// for more information
    pub fn get<I>(&self, id: I) -> Result<Sprint>
    where
        I: Into<String>,
    {
        self.jira.get("agile", &format!("/sprint/{}", id.into()))
    }

    /// Move issues into a sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/sprint-moveIssuesToSprint)
    /// for more information
    pub fn move_issues(&self, sprint_id: u64, issues: Vec<String>) -> Result<EmptyResponse> {
        let path = format!("/sprint/{sprint_id}/issue");
        let data = MoveIssues { issues };

        self.jira.post("agile", &path, data)
    }

    /// Returns a single page of sprint results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board/{boardId}/sprint-getAllSprints)
    /// for more information
    pub fn list(&self, board: &Board, options: &SearchOptions) -> Result<SprintResults> {
        let mut path = vec![format!("/board/{}/sprint", board.id)];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<SprintResults>("agile", path.join("?").as_ref())
    }

    /// Returns a type which may be used to iterate over consecutive pages of results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board/{boardId}/sprint-getAllSprints)
    /// for more information
    pub fn iter<'a>(
        &self,
        board: &'a Board,
        options: &'a SearchOptions,
    ) -> Result<SprintsIter<'a>> {
        SprintsIter::new(board, options, &self.jira)
    }

    /// Update sprint details (name, dates, state)
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-post)
    /// for more information
    pub fn update<I>(&self, id: I, data: UpdateSprint) -> Result<Sprint>
    where
        I: Into<u64>,
    {
        self.jira
            .post("agile", &format!("/sprint/{}", id.into()), data)
    }

    /// Delete a sprint
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-delete)
    /// for more information
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<u64>,
    {
        self.jira
            .delete::<EmptyResponse>("agile", &format!("/sprint/{}", id.into()))?;
        Ok(())
    }
}

/// Provides an iterator over multiple pages of search results
#[derive(Debug)]
pub struct SprintsIter<'a> {
    jira: Jira,
    board: &'a Board,
    results: SprintResults,
    search_options: &'a SearchOptions,
}

impl<'a> SprintsIter<'a> {
    fn new(board: &'a Board, options: &'a SearchOptions, jira: &Jira) -> Result<Self> {
        let results = jira.sprints().list(board, options)?;
        Ok(SprintsIter {
            board,
            jira: jira.clone(),
            results,
            search_options: options,
        })
    }

    fn more(&self) -> bool {
        !self.results.is_last
    }
}

impl Iterator for SprintsIter<'_> {
    type Item = Sprint;
    fn next(&mut self) -> Option<Sprint> {
        self.results.values.pop().or_else(|| {
            if self.more() {
                match self.jira.sprints().list(
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
                        self.results.values.pop()
                    }
                    Err(e) => {
                        tracing::error!("Sprints pagination failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncSprints {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncSprints {
    pub fn new(jira: &AsyncJira) -> AsyncSprints {
        AsyncSprints { jira: jira.clone() }
    }

    /// Create a new sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/9.5.0/#agile/1.0/sprint-createSprint)
    /// for more information
    pub async fn create<T: Into<String>>(&self, board: Board, name: T) -> Result<Sprint> {
        let data: CreateSprint = CreateSprint {
            name: name.into(),
            origin_board_id: Some(board.id),
        };
        self.jira.post("agile", "/sprint", data).await
    }

    /// Get a single sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/sprint-getSprint)
    /// for more information
    pub async fn get<I>(&self, id: I) -> Result<Sprint>
    where
        I: Into<String>,
    {
        self.jira
            .get("agile", &format!("/sprint/{}", id.into()))
            .await
    }

    /// Move issues into a sprint
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/sprint-moveIssuesToSprint)
    /// for more information
    pub async fn move_issues(&self, sprint_id: u64, issues: Vec<String>) -> Result<EmptyResponse> {
        let path = format!("/sprint/{sprint_id}/issue");
        let data = MoveIssues { issues };
        self.jira.post("agile", &path, data).await
    }

    /// Returns a single page of sprint results
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board/{boardId}/sprint-getAllSprints)
    /// for more information
    pub async fn list(&self, board: &Board, options: &SearchOptions) -> Result<SprintResults> {
        let mut path = vec![format!("/board/{}/sprint", board.id)];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();
        path.push(query);
        self.jira
            .get::<SprintResults>("agile", path.join("?").as_ref())
            .await
    }

    /// Update sprint details (name, dates, state)
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-post)
    /// for more information
    pub async fn update<I>(&self, id: I, data: UpdateSprint) -> Result<Sprint>
    where
        I: Into<u64>,
    {
        self.jira
            .post("agile", &format!("/sprint/{}", id.into()), data)
            .await
    }

    /// Delete a sprint
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-delete)
    /// for more information
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<u64>,
    {
        self.jira
            .delete::<EmptyResponse>("agile", &format!("/sprint/{}", id.into()))
            .await?;
        Ok(())
    }
}
