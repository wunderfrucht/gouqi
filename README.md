# gouqi

[![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE)
[![Released API docs](https://img.shields.io/docsrs/gouqi/latest)](http://docs.rs/gouqi)
[![Rust](https://github.com/wunderfrucht/gouqi/actions/workflows/rust.yml/badge.svg)](https://github.com/wunderfrucht/gouqi/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/wunderfrucht/gouqi/branch/main/graph/badge.svg?token=uAQXWlybzJ)](https://codecov.io/gh/wunderfrucht/gouqi)

> a rust interface for [jira](https://www.atlassian.com/software/jira)

Forked from goji <https://github.com/softprops/goji>

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
gouqi = "*"

# Optional: Enable async API
gouqi = { version = "*", features = ["async"] }
```

## usage

Please browse the [examples](examples/) directory in this repo for some example applications.

Basic usage requires a jira host, and a flavor of `jira::Credentials` for authorization.

### Synchronous API

The default API uses synchronous requests:

```rust,skeptic-template
use gouqi::{Credentials, Jira};
use std::env;
use tracing::error;

fn main() { 
    if let Ok(host) = env::var("JIRA_HOST") {
        let query = env::args().nth(1).unwrap_or("order by created DESC".to_owned());
        let jira = Jira::new(host, Credentials::Anonymous).expect("Error initializing Jira");

        match jira.search().iter(query, &Default::default()) {
            Ok(results) => {
                for issue in results {
                    println!("{:#?}", issue);
                }
            }
            Err(err) => panic!("{:#?}", err),
        }
    } else {
        error!("Missing environment variable JIRA_HOST!");
    }
}
```

### Asynchronous API

With the `async` feature enabled, you can use the asynchronous API:

```rust
use futures::stream::StreamExt;
use gouqi::{Credentials, SearchOptions};
use std::env;
use tracing::error;

#[tokio::main]
async fn main() {
    if let Ok(host) = env::var("JIRA_HOST") {
        let query = env::args().nth(1).unwrap_or("order by created DESC".to_owned());
        
        // Create an async Jira client
        let jira = gouqi::r#async::Jira::new(host, Credentials::Anonymous)
            .expect("Error initializing Jira");

        // Use the stream method to get a futures Stream
        let search_options = SearchOptions::default();
        match jira.search().stream(query, &search_options).await {
            Ok(mut stream) => {
                // Consume the stream asynchronously
                while let Some(issue) = stream.next().await {
                    println!("{:#?}", issue);
                }
            }
            Err(err) => error!("{:#?}", err),
        }
    } else {
        error!("Missing environment variable JIRA_HOST!");
    }
}
```

You can also convert between sync and async clients:

```rust
// Convert from sync to async
let sync_jira = Jira::new(host, credentials)?;
let async_jira = sync_jira.into_async();

// Convert from async to sync
let async_jira = gouqi::r#async::Jira::new(host, credentials)?;
let sync_jira = gouqi::sync::Jira::from(&async_jira);
```

## Commiting a PR

Please make sure to run `cargo fmt`, `cargo test` and `cargo clippy` before committing.
New code should contains tests.
Commits to follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

Changelog is generated using [git cliff](https://github.com/orhun/git-cliff)

```sh
cargo install git-cliff
git cliff -o --use-branch-tags
```

## what's with the name

Jira's name is a [shortened form of gojira](https://en.wikipedia.org/wiki/Jira_(software)),
another name for godzilla. Goji is a play on that.

[Goji](https://en.wikipedia.org/wiki/Goji) (Chinese: 枸杞; pinyin: gǒuqǐ)

Doug Tangren (softprops) 2016-2018
