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
                let histories = issues.changelog(issue.key).unwrap().histories;

                for history in histories {
                    let author = history.author.display_name;
                    let created = history.created;
                    let items = history.items;
                    println!("{} \t {}\n ", author, created);

                    for item in items {
                        let field = item.field;
                        let from = item.from_string.unwrap_or_else(|| "Null".to_string());
                        let to = item.to_string.unwrap_or_else(|| "Null".to_string());
                        println!("\"{}\" \t \"{}\" => \"{}\"\n", field, from, to);
                    }
                    println!()
                }
            }
            e => error!("{:?}", e),
        }
    } else {
        error!("Missing one or more environment variables JIRA_HOST, JIRA_USER, JIRA_PASS!");
    }
}
