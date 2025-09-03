# Jira V3 API Migration Guide

## Overview

This guide helps you migrate from Jira V2 to V3 API when using gouqi. The library now provides **automatic field handling** to ensure smooth transitions with zero breaking changes to your existing code.

## What Changed in Jira V3 API

### API Endpoint Changes
- **V2**: `/rest/api/2/search` 
- **V3**: `/rest/api/3/search/jql` ✅ *Automatically handled by gouqi*

### Field Behavior Changes  
- **V2**: Returns commonly used fields by default (id, self, key, fields, summary, etc.)
- **V3**: Returns **only** `id` field by default - requires explicit field specification

### Breaking Change Impact
Without field specification, V3 API responses would cause deserialization errors:
```
Error Serde(Error("missing field 'self'", line: 1, column: 25))
```

## How gouqi Handles the Migration

### Automatic Field Injection
When you use V3 API (automatically detected for `*.atlassian.net` hosts), gouqi **automatically injects essential fields** when none are explicitly specified:

```rust
// This code works seamlessly on both V2 and V3
let results = jira.search().list("project = TEST", &Default::default())?;
//                                                   ^^^^^^^^^^^^^^^^
//                                            V3: Auto-adds essential fields
//                                            V2: Uses original behavior
```

### Essential Fields Auto-Added
For V3 compatibility, gouqi automatically includes:
- `id` - Issue ID
- `self` - Issue self-link  
- `key` - Issue key (TEST-123)
- `fields` - Custom fields container

## Migration Scenarios

### ✅ Zero-Change Migration (Recommended)
Your existing code continues to work without any changes:

```rust
use gouqi::{Credentials, Jira};

// Works on both V2 and V3 automatically
let jira = Jira::new("https://company.atlassian.net", creds)?;
let results = jira.search().list("assignee = currentUser()", &Default::default())?;

for issue in results.issues {
    println!("{}: {}", issue.key, issue.summary().unwrap_or_default());
}
```

### 🎯 Explicit Field Control
For performance optimization or specific field requirements:

```rust 
use gouqi::SearchOptions;

// Minimal response (only ID)
let minimal = jira.search().list(
    "project = TEST",
    &SearchOptions::builder().minimal_fields().build()
)?;

// Essential fields for Issue struct compatibility
let essential = jira.search().list(
    "project = TEST", 
    &SearchOptions::builder().essential_fields().build()
)?;

// Commonly used fields
let standard = jira.search().list(
    "project = TEST",
    &SearchOptions::builder().standard_fields().build()
)?;

// All available fields
let complete = jira.search().list(
    "project = TEST",
    &SearchOptions::builder().all_fields().build()
)?;

// Custom field selection  
let custom = jira.search().list(
    "project = TEST",
    &SearchOptions::builder()
        .fields(vec!["id", "key", "summary", "assignee"])
        .build()
)?;
```

### 🔧 Version-Specific Configuration
Explicitly control API version if needed:

```rust
use gouqi::{Jira, SearchApiVersion};

// Force V2 (for on-premise servers)
let jira_v2 = Jira::with_search_api_version(
    "https://jira.company.com",
    creds,
    SearchApiVersion::V2,
)?;

// Force V3 (for cloud instances)
let jira_v3 = Jira::with_search_api_version(
    "https://company.atlassian.net", 
    creds,
    SearchApiVersion::V3,
)?;

// Auto-detect (default behavior)
let jira_auto = Jira::with_search_api_version(
    "https://company.atlassian.net",
    creds, 
    SearchApiVersion::Auto, // *.atlassian.net → V3, others → V2
)?;
```

## Field Selection Reference

### Available Convenience Methods

| Method | Fields Included | Use Case |
|--------|-----------------|----------|
| `.minimal_fields()` | `id` | Lightweight queries, just need issue IDs |
| `.essential_fields()` | `id, self, key, fields` | Issue struct compatibility |
| `.standard_fields()` | `id, self, key, fields, summary, status, assignee, reporter, created, updated` | Common issue operations |
| `.all_fields()` | `*all` | Complete issue data |
| `.fields(vec![...])` | Custom selection | Specific requirements |

### Performance Considerations

**Best Practice**: Use the minimal field set that meets your needs:

```rust
// ✅ Good - Only fetch what you need
let issues = jira.search().list(
    "project = TEST AND status = Open",
    &SearchOptions::builder()
        .fields(vec!["id", "key", "summary", "status"])
        .build()
)?;

// ⚠️ Acceptable - Reasonable default for most use cases
let issues = jira.search().list(
    "project = TEST",
    &SearchOptions::builder().standard_fields().build()
)?;

// ❌ Avoid - Fetches unnecessary data
let issues = jira.search().list(
    "project = TEST", 
    &SearchOptions::builder().all_fields().build()
)?;
```

## Deployment Type Detection

gouqi automatically detects your Jira deployment type:

| Host Pattern | Detected Type | Default API |
|--------------|---------------|-------------|
| `*.atlassian.net` | Jira Cloud | V3 |
| Others | On-premise | V2 |

## Async API Support

The same migration principles apply to async APIs:

```rust
use gouqi::r#async::Jira;

let jira = Jira::new("https://company.atlassian.net", creds)?;

// Auto-injection works for async too
let results = jira.search().list("project = TEST", &Default::default()).await?;
```

## Troubleshooting

### Issue: Deserialization errors with V3 API
**Symptom**: `Error("missing field 'self'", ...)`  
**Solution**: Ensure you're using gouqi v0.14.1+ with automatic field injection

### Issue: Performance degradation 
**Symptom**: Slower API responses  
**Solution**: Use specific field selection instead of auto-injection:

```rust
let options = SearchOptions::builder()
    .fields(vec!["id", "key", "summary"]) // Only fields you actually use
    .build();
```

### Issue: Missing fields in results
**Symptom**: `issue.some_field()` returns `None` unexpectedly  
**Solution**: Include the required field explicitly:

```rust  
let options = SearchOptions::builder()
    .fields(vec!["id", "key", "fields", "priority", "components"]) 
    .build();
```

## Testing Your Migration

Run your existing tests to verify compatibility:

```bash
cargo test
```

For integration testing with real Jira instances:

```rust
#[test]
fn test_v3_compatibility() {
    let jira = Jira::new("https://your-instance.atlassian.net", creds)?;
    
    // This should work without changes
    let results = jira.search().list("project = TEST ORDER BY created DESC", &Default::default())?;
    
    assert!(!results.issues.is_empty());
    assert!(results.issues[0].key.starts_with("TEST-"));
}
```

## Summary

✅ **Zero breaking changes** - existing code continues to work  
✅ **Automatic V3 field injection** - essential fields added when needed  
✅ **Explicit control available** - use convenience methods for specific needs  
✅ **Performance conscious** - fetch only essential fields by default, not `*all`  
✅ **Cloud deployment auto-detection** - V3 for `*.atlassian.net`, V2 for others

The migration is designed to be **completely transparent** for most users while providing **full control** for advanced use cases.