extern crate gouqi;

#[cfg(feature = "async")]
mod async_example {
    use futures::stream::StreamExt;
    use gouqi::{Credentials, SearchOptions};
    use std::env;
    use tracing::error;

    pub async fn run() {
        // Initialize tracing global tracing subscriber
        use tracing_subscriber::prelude::*;
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            // Use RUST_LOG environment variable to set the tracing level
            .with(tracing_subscriber::EnvFilter::from_default_env())
            // Sets this to be the default, global collector for this application.
            .init();

        if let (Ok(host), Ok(user), Ok(password)) = (
            env::var("JIRA_HOST"),
            env::var("JIRA_USER"),
            env::var("JIRA_PASS"),
        ) {
            let query = env::args()
                .nth(1)
                .unwrap_or_else(|| "order by created DESC".to_owned());

            // Create an async Jira client
            let jira = gouqi::r#async::Jira::new(host, Credentials::Basic(user, password))
                .expect("Error initializing Jira");

            // Use the stream method to get a futures Stream
            // Create search options and store the jira.search() result
            let search_options = SearchOptions::default();
            let searcher = jira.search();
            let stream_result = searcher.stream(query, &search_options).await;
            match stream_result {
                Ok(mut stream) => {
                    // Consume the stream asynchronously
                    while let Some(issue) = stream.next().await {
                        println!(
                            "{} {} ({}): reporter {} assignee {}",
                            issue.key,
                            issue.summary().unwrap_or_else(|| "???".to_owned()),
                            issue
                                .status()
                                .map(|value| value.name)
                                .unwrap_or_else(|| "???".to_owned()),
                            issue
                                .reporter()
                                .map(|value| value.display_name)
                                .unwrap_or_else(|| "???".to_owned()),
                            issue
                                .assignee()
                                .map(|value| value.display_name)
                                .unwrap_or_else(|| "???".to_owned())
                        );
                    }
                }
                Err(err) => error!("{:#?}", err),
            }
        } else {
            error!("Missing one or more environment variables JIRA_HOST, JIRA_USER, JIRA_PASS!");
        }
    }
}

// Main entry point with #[tokio::main] wrapper when async feature is enabled
#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    async_example::run().await;
}

// Fallback entry point when async feature is not enabled
#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example async_search --features async");
}
