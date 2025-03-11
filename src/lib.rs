extern crate reqwest;
extern crate serde;
extern crate tracing;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

#[cfg(feature = "async")]
extern crate futures;
#[cfg(feature = "async")]
extern crate tokio;

// Public re-exports only
// No imports needed for the root module

#[cfg(feature = "async")]
pub mod r#async;
pub mod core;
pub mod sync;

pub mod attachments;
mod builder;
pub mod components;
mod errors;
pub mod issues;
mod rep;
mod search;
mod transitions;
mod versions;

pub use crate::attachments::*;
pub use crate::builder::*;
pub use crate::components::*;
pub use crate::core::*; // Re-export all core types
pub use crate::errors::*;
pub use crate::issues::*;
pub use crate::rep::*;
#[cfg(feature = "async")]
pub use crate::search::AsyncSearch;
pub use crate::search::Search;
pub use crate::transitions::*;
pub mod boards;
pub mod resolution;
pub use crate::boards::*;
pub mod sprints;
pub use crate::sprints::*;
pub use crate::versions::*;

// Re-export the synchronous API as the default for backward compatibility
pub use sync::Jira;

#[cfg(feature = "async")]
impl crate::sync::Jira {
    /// Convert a synchronous client to an asynchronous one.
    /// Note that this requires the "async" feature to be enabled.
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
