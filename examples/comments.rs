// No extern crate needed in Rust 2024 edition

use gouqi::{Credentials, Issues, Jira};
use std::env;
use tracing::error;

fn main() {
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
        let issue_id = env::args().nth(1).unwrap_or_else(|| "KAN-1".to_owned());

        let jira =
            Jira::new(host, Credentials::Basic(user, password)).expect("Error initializing Jira");

        let issues = Issues::new(&jira);
        let issue = issues.get(issue_id);

        match issue {
            Ok(issue) => {
                if let Some(comments) = issue.comments() {
                    for comment in comments.comments {
                        println!(
                            "{:?}: {:?}: {:?}",
                            comment.author.as_ref().map(|a| &a.display_name),
                            comment.created,
                            &*comment.body, // Deref to &str
                        );
                    }
                }
            }
            e => error!("{:?}", e),
        }
    } else {
        error!("Missing one or more environment variables JIRA_HOST, JIRA_USER, JIRA_PASS!");
    }
}
