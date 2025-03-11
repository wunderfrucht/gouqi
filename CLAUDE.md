# Gouqi Development Guide

## Common Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name`
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -W clippy::missing_panics_doc`
- Code coverage: `cargo tarpaulin --out Xml --output-dir coverage --all-features -- --test-threads 1`
- Security audit: `cargo audit` (required - will fail the build if vulnerabilities are found)
- Check dependencies: `cargo udeps`
- Check dependencies and licensing: `cargo deny check bans licenses sources`
- Documentation: `cargo doc --no-deps --all-features`

## Code Style
- Use Rust 2021 edition
- Follow standard snake_case for functions/variables, CamelCase for types
- Organize imports: std lib first, then third-party, then internal modules
- Group imports by crate using `use` statements
- Error handling: use custom `Error` enum with proper error propagation
- Consistent use of `Result<T, Error>` pattern
- Document public API with doc comments (///)
- Include `# Panics` section in documentation for any function that may panic
- Use tracing for logging
- Implement proper Display and Error traits for error types
- Write tests in separate test files matching the module being tested

## Dependencies and Security
- Use rustls for TLS instead of OpenSSL (no OpenSSL dependencies allowed)
- All OpenSSL-related crates are explicitly banned in deny.toml
- Security vulnerabilities are checked with cargo-audit:
  - CI will fail if vulnerabilities are detected
  - Known false positives can be ignored with `--ignore RUSTSEC-xxxx-xxxx`
  - Regular security audits should be performed to ensure dependencies are secure
- Unicode license handling configured in deny.toml to work across different environments:
  - License confidence threshold lowered to 0.6 to accommodate CI differences
  - Support for multiple Unicode license variants (Unicode-DFS-2016, Unicode-3.0, etc.)
  - Special exceptions and clarifications for all Unicode-related crates
  - GitHub Actions uses --lenient flag for license checks

This Jira API client codebase strives for clean, idiomatic Rust with thorough documentation and testing.