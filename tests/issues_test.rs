extern crate serde_json;

use gouqi::issues::*;

#[test]
fn deserialise_issue_results() {
    let issue_results_str = r#"{
        "expand": "names,schema",
        "startAt": 0,
        "maxResults": 50,
        "total": 0,
        "issues": []
    }"#;

    let results: IssueResults = serde_json::from_str(issue_results_str).unwrap();

    assert_eq!(results.expand, Some(String::from("names,schema")));
    assert_eq!(results.start_at, 0);
    assert_eq!(results.max_results, 50);
    assert_eq!(results.total, 0);
    assert_eq!(results.issues.len(), 0);
}

mod changelog_tests {
    use super::*;
    use gouqi::Jira;
    use mockito::Server;

    #[test]
    /// https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issues/#api-rest-api-3-issue-issueidorkey-changelog-list-post
    fn test_changelog_success() {
        let mut server = Server::new();
        let url = &server.url();
        let mock_server = server
            .mock("GET", "/rest/api/latest/issue/TEST-1/changelog")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
  "values": [
    {
      "author": {
        "accountId": "5b10a2844c20165700ede21g",
        "active": true,
        "avatarUrls": {
          "16x16": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=16&s=16",
          "24x24": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=24&s=24",
          "32x32": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=32&s=32",
          "48x48": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=48&s=48"
        },
        "displayName": "Mia Krystof",
        "emailAddress": "mia@example.com",
        "self": "https://your-domain.atlassian.net/rest/api/3/user?accountId=5b10a2844c20165700ede21g",
        "timeZone": "Australia/Sydney"
      },
      "created": "1970-01-18T06:27:50.429+0000",
      "id": "10001",
      "items": [
        {
          "field": "fields",
          "fieldtype": "jira",
          "fieldId": "fieldId",
          "from": null,
          "fromString": "",
          "to": null,
          "toString": "label-1"
        }
      ]
    },
    {
      "author": {
        "accountId": "5b10a2844c20165700ede21g",
        "active": true,
        "avatarUrls": {
          "16x16": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=16&s=16",
          "24x24": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=24&s=24",
          "32x32": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=32&s=32",
          "48x48": "https://avatar-management--avatars.server-location.prod.public.atl-paas.net/initials/MK-5.png?size=48&s=48"
        },
        "displayName": "Mia Krystof",
        "emailAddress": "mia@example.com",
        "self": "https://your-domain.atlassian.net/rest/api/3/user?accountId=5b10a2844c20165700ede21g",
        "timeZone": "Australia/Sydney"
      },
      "created": "1970-01-18T06:27:51.429+0000",
      "id": "10002",
      "items": [
        {
          "field": "fields",
          "fieldtype": "jira",
          "fieldId": "fieldId",
          "from": null,
          "fromString": "label-1",
          "to": null,
          "toString": "label-1 label-2"
        }
      ]
    }
  ],
  "maxResults": 2,
  "startAt": 0,
  "total": 2
}"#)
            .create();

        let jira = Jira::new(&*url, gouqi::Credentials::Anonymous).unwrap();
        let issues = Issues::new(&jira);

        let result = issues.changelog("TEST-1");
        mock_server.assert();
        assert_eq!(result.is_ok(), true);
        let changelog = result.unwrap();
        assert_eq!(changelog.histories.len(), 2);
        assert_eq!(
            changelog.histories[0].created,
            "1970-01-18T06:27:50.429+0000"
        );
        assert_eq!(
            changelog.histories[0].author.display_name,
            "Mia Krystof".to_string()
        );
    }

    #[test]
    fn test_changelog_not_found() {
        let mut server = Server::new();
        let url = &server.url();
        let mock_server = server
            .mock("GET", "/rest/api/latest/issue/NONEXISTENT-1/changelog")
            .with_status(404)
            .create();

        let jira = Jira::new(&*url, gouqi::Credentials::Anonymous).unwrap();
        let issues = Issues::new(&jira);

        let result = issues.changelog("NONEXISTENT-1");

        assert!(result.is_err());

        mock_server.assert();
    }

    #[test]
    fn test_changelog_server_error() {
        let mut server = Server::new();
        let url = &server.url();
        let mock_server = server
            .mock("GET", "/rest/api/latest/issue/TEST-2/changelog")
            .with_status(500)
            .create();

        let jira = Jira::new(&*url, gouqi::Credentials::Anonymous).unwrap();
        let issues = Issues::new(&jira);

        let result = issues.changelog("TEST-2");

        assert!(result.is_err());

        mock_server.assert();
    }
}
