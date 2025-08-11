//! Interfaces for searching for issues

// Third party
use url::form_urlencoded;

// Ours
use crate::core::PaginationInfo;
use crate::sync::Jira;
use crate::{Issue, Result, SearchOptions, SearchResults};

#[cfg(feature = "async")]
use futures::Future;
#[cfg(feature = "async")]
use futures::stream::Stream;
#[cfg(feature = "async")]
use std::pin::Pin;

/// Search interface
#[derive(Debug)]
pub struct Search {
    jira: Jira,
}

impl Search {
    pub fn new(jira: &Jira) -> Search {
        Search { jira: jira.clone() }
    }

    /// Returns a single page of search results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub fn list<J>(&self, jql: J, options: &SearchOptions) -> Result<SearchResults>
    where
        J: Into<String>,
    {
        let mut path = vec!["/search".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options)
            .append_pair("jql", &jql.into())
            .finish();
        path.push(query);
        self.jira
            .get::<SearchResults>("api", path.join("?").as_ref())
    }

    /// Return a type which may be used to iterate over consecutive pages of results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub fn iter<'a, J>(&self, jql: J, options: &'a SearchOptions) -> Result<Iter<'a>>
    where
        J: Into<String>,
    {
        Iter::new(jql, options, &self.jira)
    }
}

/// Provides an iterator over multiple pages of search results
#[derive(Debug)]
pub struct Iter<'a> {
    jira: Jira,
    jql: String,
    results: SearchResults,
    search_options: &'a SearchOptions,
}

impl PaginationInfo for Iter<'_> {}

impl<'a> Iter<'a> {
    fn new<J>(jql: J, options: &'a SearchOptions, jira: &Jira) -> Result<Self>
    where
        J: Into<String>,
    {
        let query = jql.into();
        let results = jira.search().list(query.clone(), options)?;
        Ok(Iter {
            jira: jira.clone(),
            jql: query,
            results,
            search_options: options,
        })
    }

    fn more(&self) -> bool {
        let start_at = self.results.start_at;
        let current_count = self.results.issues.len() as u64;
        let total = self.results.total;
        Self::more_pages(current_count, start_at, total)
    }
}

impl Iterator for Iter<'_> {
    type Item = Issue;
    fn next(&mut self) -> Option<Issue> {
        self.results.issues.pop().or_else(|| {
            if self.more() {
                match self.jira.search().list(
                    self.jql.clone(),
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
/// Async version of the Search interface
#[derive(Debug)]
pub struct AsyncSearch {
    jira: crate::r#async::Jira,
}

#[cfg(feature = "async")]
impl AsyncSearch {
    pub fn new(jira: &crate::r#async::Jira) -> Self {
        AsyncSearch { jira: jira.clone() }
    }

    /// Returns a single page of search results asynchronously
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub async fn list<J>(&self, jql: J, options: &SearchOptions) -> Result<SearchResults>
    where
        J: Into<String>,
    {
        let mut path = vec!["/search".to_owned()];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options)
            .append_pair("jql", &jql.into())
            .finish();
        path.push(query);
        self.jira
            .get::<SearchResults>("api", path.join("?").as_ref())
            .await
    }

    /// Return a stream which yields issues from consecutive pages of results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub async fn stream<'a, J>(
        &'a self,
        jql: J,
        options: &'a SearchOptions,
    ) -> Result<impl Stream<Item = Issue> + 'a>
    where
        J: Into<String> + Clone + 'a,
    {
        let jql_string = jql.into();
        let initial_results = self.list(jql_string.clone(), options).await?;

        let stream = AsyncIssueStream {
            jira: self,
            jql: jql_string,
            search_options: options,
            current_results: initial_results,
            current_index: 0,
        };

        Ok(stream)
    }
}

#[cfg(feature = "async")]
struct AsyncIssueStream<'a> {
    jira: &'a AsyncSearch,
    jql: String,
    search_options: &'a SearchOptions,
    current_results: SearchResults,
    current_index: usize,
}

#[cfg(feature = "async")]
impl PaginationInfo for AsyncIssueStream<'_> {}

#[cfg(feature = "async")]
impl Stream for AsyncIssueStream<'_> {
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
        let more_pages = Self::more_pages(
            self.current_results.issues.len() as u64,
            self.current_results.start_at,
            self.current_results.total,
        );

        if more_pages {
            // Create a future to fetch the next page
            let jira = self.jira;
            let jql = self.jql.clone();
            let next_options = self
                .search_options
                .as_builder()
                .max_results(self.current_results.max_results)
                .start_at(self.current_results.start_at + self.current_results.max_results)
                .build();

            let future = async move { jira.list(jql, &next_options).await };

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
