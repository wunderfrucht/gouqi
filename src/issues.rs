//! Interfaces for accessing and managing issues

// Third party
use serde::Serialize;
use std::collections::BTreeMap;
use url::form_urlencoded;

// Ours
use crate::sync::Jira;
use crate::{
    Board, Changelog, Comment, Issue, IssueType, Priority, Project, Result, SearchOptions,
};

#[cfg(feature = "async")]
use futures::stream::Stream;
#[cfg(feature = "async")]
use futures::Future;
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
pub struct Component {
    pub id: String,
    pub name: String,
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
