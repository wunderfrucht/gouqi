//! Rust interface for Jira API.
//!
//! This crate provides both synchronous and asynchronous clients for interacting with Jira's REST API.
//!
//! # Features
//!
//! - Synchronous API (default)
//! - Asynchronous API (with the `async` feature)
//! - Support for multiple authentication methods:
//!   - Anonymous
//!   - Basic authentication (username/password or Personal Access Token)
//!   - Bearer token authentication
//!   - Cookie-based authentication (JSESSIONID)
//!
//! # Examples
//!
//! ## Synchronous usage
//!
//! ```no_run
//! use gouqi::{Credentials, Jira};
//!
//! let credentials = Credentials::Basic("username".to_string(), "password".to_string());
//! let jira = Jira::new("https://jira.example.com", credentials).unwrap();
//!
//! // Get information about the current session
//! let session = jira.session().unwrap();
//! println!("Logged in as: {}", session.name);
//!
//! // Search for issues
//! let results = jira.search().list("project = DEMO", &Default::default()).unwrap();
//! println!("Found {} issues", results.total);
//! ```
//!
//! ## Asynchronous usage (with the `async` feature)
//!
//! ```no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use gouqi::{Credentials, r#async::Jira};
//!
//! let credentials = Credentials::Basic("username".to_string(), "password".to_string());
//! let jira = Jira::new("https://jira.example.com", credentials)?;
//!
//! // Get information about the current session
//! let session = jira.session().await?;
//! println!("Logged in as: {}", session.name);
//!
//! // Search for issues
//! let results = jira.search().list("project = DEMO", &Default::default()).await?;
//! println!("Found {} issues", results.total);
//! # Ok(())
//! # }
//! ```

// No extern crate needed in Rust 2024 edition - dependencies are automatically available

// Public re-exports only
// No imports needed for the root module

#[cfg(feature = "async")]
pub mod r#async;
pub mod core;
pub mod sync;

pub mod attachments;
mod builder;
#[cfg(feature = "cache")]
pub mod cache;
pub mod components;
pub mod config;
pub mod env;
mod errors;
pub mod issue_links;
pub mod issues;
pub mod mcp;
#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "oauth")]
pub mod oauth;
#[cfg(any(feature = "metrics", feature = "cache"))]
pub mod observability;
pub mod projects;
pub mod relationships;
mod rep;
mod search;
pub mod transitions;
pub mod versions;

pub use crate::attachments::*;
pub use crate::builder::*;
#[cfg(feature = "cache")]
pub use crate::cache::*;
pub use crate::components::*;
pub use crate::config::*;
pub use crate::core::*; // Re-export all core types
pub use crate::errors::*;
pub use crate::issue_links::*;
pub use crate::issues::*;
#[cfg(feature = "metrics")]
pub use crate::metrics::*;
#[cfg(any(feature = "metrics", feature = "cache"))]
pub use crate::observability::*;
pub use crate::projects::*;
pub use crate::relationships::*;
pub use crate::rep::*;
#[cfg(feature = "async")]
pub use crate::search::AsyncSearch;
pub use crate::search::Search;
pub use crate::transitions::*;
pub mod boards;
pub mod resolution;
pub use crate::boards::*;
pub mod sprints;
pub use crate::mcp::*;
pub use crate::sprints::*;
pub use crate::versions::*;

// Re-export the synchronous API as the default for backward compatibility
pub use sync::Jira;

#[cfg(feature = "async")]
impl crate::sync::Jira {
    /// Convert a synchronous client to an asynchronous one.
    /// Note that this requires the "async" feature to be enabled.
    ///
    /// # Panics
    ///
    /// This function will panic if it cannot create a new async Jira client
    /// from the core configuration. This should never happen in practice since
    /// we're reusing an already validated core configuration.
    pub fn into_async(&self) -> crate::r#async::Jira {
        // Using the ClientCore directly is more reliable than trying to recreate it
        if let Ok(jira) = crate::r#async::Jira::with_core(self.core.clone()) {
            jira
        } else {
            // This fallback should never be needed since we're reusing the core
            crate::r#async::Jira::new("http://localhost", Credentials::Anonymous).unwrap()
        }
    }
}
