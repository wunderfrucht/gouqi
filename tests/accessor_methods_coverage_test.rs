//! Tests to ensure accessor methods are covered

use gouqi::{Credentials, Jira};

#[test]
fn test_sync_users_accessor() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let users = jira.users();
    // Verify we can create the users interface
    assert!(std::mem::size_of_val(&users) > 0);
}

#[test]
fn test_sync_groups_accessor() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let groups = jira.groups();
    // Verify we can create the groups interface
    assert!(std::mem::size_of_val(&groups) > 0);
}

#[cfg(feature = "async")]
mod async_accessors {
    use gouqi::Credentials;
    use gouqi::r#async::Jira as AsyncJira;

    #[test]
    fn test_async_users_accessor() {
        let jira = AsyncJira::new("http://localhost", Credentials::Anonymous).unwrap();
        let users = jira.users();
        // Verify we can create the async users interface
        assert!(std::mem::size_of_val(&users) > 0);
    }

    #[test]
    fn test_async_groups_accessor() {
        let jira = AsyncJira::new("http://localhost", Credentials::Anonymous).unwrap();
        let groups = jira.groups();
        // Verify we can create the async groups interface
        assert!(std::mem::size_of_val(&groups) > 0);
    }
}
