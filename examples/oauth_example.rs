//! OAuth 1.0a Authentication Example for Jira Server/Data Center
//!
//! This example demonstrates how to use OAuth 1.0a authentication with gouqi
//! for Jira Server or Data Center deployments.
//!
//! # Prerequisites
//!
//! Before using OAuth 1.0a with Jira, you need to:
//!
//! 1. **Generate an RSA key pair**:
//!    ```bash
//!    openssl genrsa -out jira_privatekey.pem 2048
//!    openssl rsa -in jira_privatekey.pem -pubout -out jira_publickey.pem
//!    ```
//!
//! 2. **Register your application in Jira**:
//!    - Go to Jira Administration → Application Links
//!    - Create a new Application Link with your application URL
//!    - Configure it as a Generic Application
//!    - Set up incoming authentication with your public key
//!    - Note the Consumer Key you choose
//!
//! 3. **Perform the OAuth dance** (3-legged OAuth flow):
//!    - Request a request token
//!    - Direct user to authorization URL
//!    - Exchange authorized request token for access token
//!    - Save the access token and access token secret
//!
//! # Running This Example
//!
//! This example assumes you've already completed the OAuth flow and have:
//! - Consumer key
//! - RSA private key (PEM format)
//! - Access token
//! - Access token secret
//!
//! Set the following environment variables:
//! ```bash
//! export JIRA_HOST="https://your-jira-server.com"
//! export JIRA_OAUTH_CONSUMER_KEY="your-consumer-key"
//! export JIRA_OAUTH_PRIVATE_KEY_PATH="/path/to/jira_privatekey.pem"
//! export JIRA_OAUTH_ACCESS_TOKEN="your-access-token"
//! export JIRA_OAUTH_ACCESS_TOKEN_SECRET="your-access-token-secret"
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example oauth_example --features oauth
//! ```
//!
//! # OAuth 2.0 for Jira Cloud
//!
//! Note: Jira Cloud uses OAuth 2.0, not OAuth 1.0a. For OAuth 2.0, once you have
//! an access token from the OAuth flow, use it with `Credentials::Bearer`:
//!
//! ```no_run
//! use gouqi::{Credentials, Jira};
//!
//! let credentials = Credentials::Bearer("your-oauth2-access-token".to_string());
//! let jira = Jira::new("https://your-domain.atlassian.net", credentials).unwrap();
//! ```

#[cfg(feature = "oauth")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gouqi::{Credentials, Jira};
    use std::env;
    use std::fs;

    // Load configuration from environment
    let host = env::var("JIRA_HOST").expect("JIRA_HOST environment variable not set");
    let consumer_key = env::var("JIRA_OAUTH_CONSUMER_KEY")
        .expect("JIRA_OAUTH_CONSUMER_KEY environment variable not set");
    let private_key_path = env::var("JIRA_OAUTH_PRIVATE_KEY_PATH")
        .expect("JIRA_OAUTH_PRIVATE_KEY_PATH environment variable not set");
    let access_token = env::var("JIRA_OAUTH_ACCESS_TOKEN")
        .expect("JIRA_OAUTH_ACCESS_TOKEN environment variable not set");
    let access_token_secret = env::var("JIRA_OAUTH_ACCESS_TOKEN_SECRET")
        .expect("JIRA_OAUTH_ACCESS_TOKEN_SECRET environment variable not set");

    // Read the private key from file
    let private_key_pem =
        fs::read_to_string(&private_key_path).expect("Failed to read private key file");

    println!("Setting up OAuth 1.0a authentication...");
    println!("Host: {}", host);
    println!("Consumer Key: {}", consumer_key);
    println!("Private Key Path: {}", private_key_path);

    // Create OAuth 1.0a credentials
    let credentials = Credentials::OAuth1a {
        consumer_key,
        private_key_pem,
        access_token,
        access_token_secret,
    };

    // Create Jira client with OAuth credentials
    let jira = Jira::new(host, credentials)?;

    println!("\nTesting OAuth authentication...");

    // Test the connection by getting session information
    match jira.session() {
        Ok(session) => {
            println!("✓ Authentication successful!");
            println!("  Logged in as: {}", session.name);
        }
        Err(e) => {
            eprintln!("✗ Authentication failed: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Verify your OAuth credentials are correct");
            eprintln!("2. Ensure the access token hasn't expired");
            eprintln!("3. Check that the consumer key matches your Jira application link");
            eprintln!("4. Verify the private key is in correct PEM format");
            return Err(e.into());
        }
    }

    // Example: Search for issues
    println!("\nSearching for issues...");
    match jira
        .search()
        .list("order by created DESC", &Default::default())
    {
        Ok(results) => {
            println!("✓ Found {} issues", results.total);
            if let Some(first_issue) = results.issues.first() {
                let summary = first_issue
                    .summary()
                    .unwrap_or_else(|| "No summary".to_string());
                println!("  Latest issue: {} - {}", first_issue.key, summary);
            }
        }
        Err(e) => {
            eprintln!("✗ Search failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "oauth"))]
fn main() {
    eprintln!("This example requires the 'oauth' feature to be enabled.");
    eprintln!("Run with: cargo run --example oauth_example --features oauth");
    std::process::exit(1);
}
