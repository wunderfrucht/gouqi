use gouqi::{Credentials, Jira, Result};

#[test]
fn test_attachments_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _attachments = jira.attachments();

    // Just verify we can create an Attachments instance
    // Simply checking that it works without panic
    Ok(())
}

#[test]
fn test_components_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _components = jira.components();

    // Just verify we can create a Components instance without panic
    Ok(())
}

#[test]
fn test_boards_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _boards = jira.boards();

    // Just verify we can create a Boards instance without panic
    Ok(())
}

#[test]
fn test_sprints_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _sprints = jira.sprints();

    // Just verify we can create a Sprints instance without panic
    Ok(())
}

#[test]
fn test_versions_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _versions = jira.versions();

    // Just verify we can create a Versions instance without panic
    Ok(())
}

// We can't easily test session without mocking, so we'll skip a real test here

#[test]
fn test_transitions_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _transitions = jira.transitions("ISSUE-123");

    // Just verify we can create a Transitions instance without panic
    Ok(())
}

#[test]
fn test_issues_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _issues = jira.issues();

    // Just verify we can create an Issues instance without panic
    Ok(())
}

#[test]
fn test_search_method() -> Result<()> {
    let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
    let _search = jira.search();

    // Just verify we can create a Search instance without panic
    Ok(())
}

#[test]
fn test_from_client_method() -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let _jira = Jira::from_client("http://example.com", Credentials::Anonymous, client)?;

    // Just verify we can create a Jira instance with a custom client
    Ok(())
}
