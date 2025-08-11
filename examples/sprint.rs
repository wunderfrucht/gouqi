// No extern crate needed in Rust 2024 edition

use gouqi::{Credentials, Jira, Sprints};
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
        let sprint_id = env::args().nth(1).unwrap_or_else(|| "1".to_owned());

        let jira =
            Jira::new(host, Credentials::Basic(user, password)).expect("Error initializing Jira");

        let sprints = Sprints::new(&jira);

        match sprints.get(sprint_id) {
            Ok(sprint) => println!("{sprint:?}"),
            e => error!("{:?}", e),
        }
    } else {
        error!("Missing one or more environment variables JIRA_HOST, JIRA_USER, JIRA_PASS!");
    }
}
