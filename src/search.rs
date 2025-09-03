//! Interfaces for searching for issues

// Third party
use tracing::warn;
use url::form_urlencoded;

// Ours
use crate::core::PaginationInfo;
use crate::sync::Jira;
use crate::{Error, Issue, Result, SearchOptions, SearchResults, V3SearchResults};

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

    /// Validate query for V3 API requirements
    /// V3 requires "bounded" queries which means having maxResults set
    fn validate_v3_query(jql: &str, options: &SearchOptions) -> Result<()> {
        // V3 requires maxResults to be set (bounded query requirement)
        if options.max_results().is_none() {
            return Err(Error::InvalidQuery {
                message:
                    "V3 API requires bounded queries. Please set maxResults parameter (max 5000)."
                        .to_string(),
            });
        }

        // Empty JQL is not allowed in V3
        if jql.trim().is_empty() {
            return Err(Error::InvalidQuery {
                message:
                    "V3 API does not allow empty JQL queries. Please provide a valid JQL query."
                        .to_string(),
            });
        }

        // Provide helpful warning for potentially expensive queries
        // This is not a hard requirement but helps users write better queries
        let jql_lower = jql.to_lowercase();
        let has_limiting_clause = jql_lower.contains("project")
            || jql_lower.contains("assignee")
            || jql_lower.contains("reporter")
            || jql_lower.contains("created")
            || jql_lower.contains("updated")
            || jql_lower.contains("key")
            || jql_lower.contains("id")
            || jql_lower.contains("sprint")
            || jql_lower.contains("fixversion")
            || jql_lower.contains("component");

        if !has_limiting_clause {
            // This is just a warning in logs, not an error
            // The API will handle the actual validation
            warn!(
                "JQL query may be expensive without limiting clauses: {}",
                jql
            );
        }

        Ok(())
    }

    /// Get the appropriate API name and endpoint based on configured search API version
    pub fn get_search_endpoint(&self) -> (&'static str, &'static str, Option<&'static str>) {
        use crate::core::SearchApiVersion;
        match self.jira.core.get_search_api_version() {
            SearchApiVersion::V2 => ("api", "/search", Some("latest")),
            SearchApiVersion::V3 => ("api", "/search/jql", Some("3")),
            SearchApiVersion::Auto => {
                // This should not happen as get_search_api_version resolves Auto to a concrete version
                ("api", "/search", Some("latest"))
            }
        }
    }

    /// Returns a single page of search results
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub fn list<J>(&self, jql: J, options: &SearchOptions) -> Result<SearchResults>
    where
        J: Into<String>,
    {
        let jql_string = jql.into();
        let (api_name, endpoint, version) = self.get_search_endpoint();

        // Auto-inject maxResults for V3 if not set to meet bounded query requirement
        let mut final_options = options.clone();
        if matches!(
            self.jira.core.get_search_api_version(),
            crate::core::SearchApiVersion::V3
        ) {
            // Ensure maxResults is set for V3 (bounded query requirement)
            let current_max = final_options.max_results();
            if current_max.is_none() {
                final_options = final_options
                    .as_builder()
                    .max_results(50) // Default to 50 if not specified
                    .build();
            } else if let Some(max) = current_max {
                // V3 API has a maximum limit of 5000 results per request
                if max > 5000 {
                    warn!(
                        "maxResults {} exceeds V3 API limit of 5000, capping at 5000",
                        max
                    );
                    final_options = final_options.as_builder().max_results(5000).build();
                }
            }

            // Validate V3 requirements
            Self::validate_v3_query(&jql_string, &final_options)?;
        }

        let mut path = vec![endpoint.to_owned()];

        // Auto-inject essential fields for V3 if none specified
        if !final_options.fields_explicitly_set()
            && matches!(
                self.jira.core.get_search_api_version(),
                crate::core::SearchApiVersion::V3
            )
        {
            final_options = final_options.as_builder().essential_fields().build();
        }

        let query_options = final_options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options)
            .append_pair("jql", &jql_string)
            .finish();
        path.push(query);

        // Handle different response formats for V2 vs V3
        match self.jira.core.get_search_api_version() {
            crate::core::SearchApiVersion::V3 => {
                // Use new V3SearchResults format and convert to legacy format
                let v3_results = self.jira.get_versioned::<V3SearchResults>(
                    api_name,
                    version,
                    path.join("?").as_ref(),
                )?;

                // Extract pagination info from options for conversion
                let start_at = final_options.start_at().unwrap_or(0);
                let max_results = final_options.max_results().unwrap_or(50);

                Ok(v3_results.to_search_results(start_at, max_results))
            }
            _ => {
                // Use legacy SearchResults format for V2
                let mut results = self.jira.get_versioned::<SearchResults>(
                    api_name,
                    version,
                    path.join("?").as_ref(),
                )?;
                // V2 provides accurate totals
                results.total_is_accurate = Some(true);
                results.is_last_page =
                    Some(results.start_at + results.issues.len() as u64 >= results.total);
                Ok(results)
            }
        }
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
        // For V3 API, use is_last_page if available
        if let Some(is_last) = self.results.is_last_page {
            return !is_last;
        }

        // For V2 API or fallback, use traditional pagination logic
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
                // Build options for next page
                let next_options = if let Some(ref token) = self.results.next_page_token {
                    // V3 API: Use nextPageToken for pagination
                    self.search_options
                        .as_builder()
                        .max_results(self.results.max_results)
                        .next_page_token(token)
                        .build()
                } else {
                    // V2 API: Use startAt for pagination
                    self.search_options
                        .as_builder()
                        .max_results(self.results.max_results)
                        .start_at(self.results.start_at + self.results.max_results)
                        .build()
                };

                match self.jira.search().list(self.jql.clone(), &next_options) {
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

    /// Get the appropriate API name and endpoint based on configured search API version
    pub fn get_search_endpoint(&self) -> (&'static str, &'static str, Option<&'static str>) {
        use crate::core::SearchApiVersion;
        match self.jira.core.get_search_api_version() {
            SearchApiVersion::V2 => ("api", "/search", Some("latest")),
            SearchApiVersion::V3 => ("api", "/search/jql", Some("3")),
            SearchApiVersion::Auto => {
                // This should not happen as get_search_api_version resolves Auto to a concrete version
                ("api", "/search", Some("latest"))
            }
        }
    }

    /// Returns a single page of search results asynchronously
    ///
    /// See the [jira docs](https://docs.atlassian.com/jira/REST/latest/#api/2/search)
    /// for more information
    pub async fn list<J>(&self, jql: J, options: &SearchOptions) -> Result<SearchResults>
    where
        J: Into<String>,
    {
        let jql_string = jql.into();
        let (api_name, endpoint, version) = self.get_search_endpoint();

        // Auto-inject maxResults for V3 if not set to meet bounded query requirement
        let mut final_options = options.clone();
        if matches!(
            self.jira.core.get_search_api_version(),
            crate::core::SearchApiVersion::V3
        ) {
            // Ensure maxResults is set for V3 (bounded query requirement)
            let current_max = final_options.max_results();
            if current_max.is_none() {
                final_options = final_options
                    .as_builder()
                    .max_results(50) // Default to 50 if not specified
                    .build();
            } else if let Some(max) = current_max {
                // V3 API has a maximum limit of 5000 results per request
                if max > 5000 {
                    warn!(
                        "maxResults {} exceeds V3 API limit of 5000, capping at 5000",
                        max
                    );
                    final_options = final_options.as_builder().max_results(5000).build();
                }
            }

            // Validate V3 requirements
            Search::validate_v3_query(&jql_string, &final_options)?;
        }

        let mut path = vec![endpoint.to_owned()];

        // Auto-inject essential fields for V3 if none specified
        if !final_options.fields_explicitly_set()
            && matches!(
                self.jira.core.get_search_api_version(),
                crate::core::SearchApiVersion::V3
            )
        {
            final_options = final_options.as_builder().essential_fields().build();
        }

        let query_options = final_options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options)
            .append_pair("jql", &jql_string)
            .finish();
        path.push(query);

        // Handle different response formats for V2 vs V3
        match self.jira.core.get_search_api_version() {
            crate::core::SearchApiVersion::V3 => {
                // Use new V3SearchResults format and convert to legacy format
                let v3_results = self
                    .jira
                    .get_versioned::<V3SearchResults>(api_name, version, path.join("?").as_ref())
                    .await?;

                // Extract pagination info from options for conversion
                let start_at = final_options.start_at().unwrap_or(0);
                let max_results = final_options.max_results().unwrap_or(50);

                Ok(v3_results.to_search_results(start_at, max_results))
            }
            _ => {
                // Use legacy SearchResults format for V2
                let mut results = self
                    .jira
                    .get_versioned::<SearchResults>(api_name, version, path.join("?").as_ref())
                    .await?;
                // V2 provides accurate totals
                results.total_is_accurate = Some(true);
                results.is_last_page =
                    Some(results.start_at + results.issues.len() as u64 >= results.total);
                Ok(results)
            }
        }
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
