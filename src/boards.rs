//! Interfaces for accessing and managing boards

// Third party
use serde::Deserialize;
use url::form_urlencoded;

// Ours
use crate::{Jira, Result, SearchOptions};

#[cfg(feature = "async")]
use futures::stream::Stream;
#[cfg(feature = "async")]
use std::marker::PhantomData;
#[cfg(feature = "async")]
use std::pin::Pin;
#[cfg(feature = "async")]
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct Boards {
    jira: Jira,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Board {
    #[serde(rename = "self")]
    pub self_link: String,
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub location: Option<Location>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(rename = "projectId")]
    pub project_id: Option<u64>,
    #[serde(rename = "userId")]
    pub user_id: Option<u64>,
    #[serde(rename = "userAccountId")]
    pub user_account_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "projectKey")]
    pub project_key: Option<String>,
    #[serde(rename = "projectTypeKey")]
    pub project_type_key: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BoardResults {
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub values: Vec<Board>,
}

impl Boards {
    pub fn new(jira: &Jira) -> Boards {
        Boards { jira: jira.clone() }
    }

    /// Get a single board
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getBoard)
    /// for more information
    pub fn get<I>(&self, id: I) -> Result<Board>
    where
        I: Into<u64>,
    {
        self.jira.get("agile", &format!("/board/{}", id.into()))
    }

    /// Returns a single page of board results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getAllBoards)
    /// for more information
    pub fn list(&self, options: &SearchOptions) -> Result<BoardResults> {
        let mut path = vec!["/board".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<BoardResults>("agile", path.join("?").as_ref())
    }

    /// Returns a type which may be used to iterate over consecutive pages of results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getAllBoards)
    /// for more information
    pub fn iter<'a>(&self, options: &'a SearchOptions) -> Result<BoardsIter<'a>> {
        BoardsIter::new(options, &self.jira)
    }
}

/// Provides an iterator over multiple pages of search results
#[derive(Debug)]
pub struct BoardsIter<'a> {
    jira: Jira,
    results: BoardResults,
    search_options: &'a SearchOptions,
}

impl<'a> BoardsIter<'a> {
    fn new(options: &'a SearchOptions, jira: &Jira) -> Result<Self> {
        let results = jira.boards().list(options)?;
        Ok(BoardsIter {
            jira: jira.clone(),
            results,
            search_options: options,
        })
    }

    fn more(&self) -> bool {
        !self.results.is_last
    }
}

impl Iterator for BoardsIter<'_> {
    type Item = Board;
    fn next(&mut self) -> Option<Board> {
        self.results.values.pop().or_else(|| {
            if self.more() {
                match self.jira.boards().list(
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
                        tracing::error!("Boards pagination failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}

/// Asynchronous interface for accessing and managing boards
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncBoards<'a> {
    jira: &'a crate::r#async::Jira,
}

#[cfg(feature = "async")]
impl<'a> AsyncBoards<'a> {
    /// Creates a new AsyncBoards instance
    pub fn new(jira: &'a crate::r#async::Jira) -> Self {
        AsyncBoards { jira }
    }

    /// Get a single board asynchronously
    ///
    /// See this [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getBoard)
    /// for more information
    pub async fn get<I>(&self, id: I) -> Result<Board>
    where
        I: Into<u64>,
    {
        self.jira
            .get("agile", &format!("/board/{}", id.into()))
            .await
    }

    /// Returns a single page of board results asynchronously
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getAllBoards)
    /// for more information
    pub async fn list(&self, options: &SearchOptions) -> Result<BoardResults> {
        let mut path = vec!["/board".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<BoardResults>("agile", path.join("?").as_ref())
            .await
    }

    /// Returns a stream which can be used to asynchronously iterate over pages of results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getAllBoards)
    /// for more information
    pub async fn stream<'b>(
        self: &'a AsyncBoards<'a>,
        options: &'b SearchOptions,
    ) -> Result<AsyncBoardsStream<'a, 'b>> {
        let results = self.list(options).await?;
        Ok(AsyncBoardsStream {
            boards: self,
            current_results: results,
            search_options: options,
            index: 0,
            _phantom: PhantomData,
        })
    }
}

/// Provides a stream over multiple pages of board search results
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncBoardsStream<'a, 'b> {
    boards: &'a AsyncBoards<'a>,
    current_results: BoardResults,
    search_options: &'b SearchOptions,
    index: usize,
    _phantom: PhantomData<&'a ()>,
}

#[cfg(feature = "async")]
impl AsyncBoardsStream<'_, '_> {
    fn more(&self) -> bool {
        !self.current_results.is_last
    }
}

#[cfg(feature = "async")]
impl Stream for AsyncBoardsStream<'_, '_> {
    type Item = Result<Board>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If we have items remaining in the current page
        if self.index < self.current_results.values.len() {
            let board = self.current_results.values[self.index].clone();
            self.index += 1;
            return Poll::Ready(Some(Ok(board)));
        }

        // If there are more pages to load
        if self.more() {
            use std::future::Future;

            // We need to handle this differently due to the self reference
            // Create a mutable reference to our fields
            let this = &mut *self;

            // Create a future that doesn't capture self
            let next_options = this
                .search_options
                .as_builder()
                .max_results(this.current_results.max_results)
                .start_at(this.current_results.start_at + this.current_results.max_results)
                .build();

            // This is a temporary work-around since we can't easily do async in poll_next
            // In a real implementation, we'd use a better pattern for this
            let mut future = Box::pin(this.boards.list(&next_options));

            match Future::poll(future.as_mut(), cx) {
                Poll::Ready(Ok(new_results)) => {
                    this.current_results = new_results;
                    this.index = 0;

                    if !this.current_results.values.is_empty() {
                        let board = this.current_results.values[this.index].clone();
                        this.index += 1;
                        Poll::Ready(Some(Ok(board)))
                    } else {
                        Poll::Ready(None)
                    }
                }
                Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
                Poll::Pending => Poll::Pending,
            }
        } else {
            // No more items
            Poll::Ready(None)
        }
    }
}
