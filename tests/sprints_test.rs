// No extern crate needed in Rust 2024 edition

use gouqi::sprints::*;
use time::macros::datetime;

#[test]
fn deserialise_sprint() {
    let sprint_str = r#"{
        "id": 72,
        "self": "http://www.example.com/jira/rest/agile/1.0/sprint/73",
        "name": "sprint 2"
    }"#;

    let sprint: Sprint = serde_json::from_str(sprint_str).unwrap();

    assert_eq!(sprint.id, 72u64);
    assert_eq!(sprint.name, "sprint 2");
    assert_eq!(
        sprint.self_link,
        "http://www.example.com/jira/rest/agile/1.0/sprint/73"
    );
    assert_eq!(sprint.state, None);
    assert_eq!(sprint.start_date, None);
    assert_eq!(sprint.end_date, None);
    assert_eq!(sprint.complete_date, None);
    assert_eq!(sprint.origin_board_id, None);
}

#[test]
fn deserialise_sprint_with_optional_fields() {
    let sprint_str = r#"{
        "id": 72,
        "self": "http://www.example.com/jira/rest/agile/1.0/sprint/73",
        "state": "future",
        "name": "sprint 2",
        "startDate": "2015-04-11T15:22:00.000+10:00",
        "endDate": "2015-04-20T01:22:00.000+10:00",
        "completeDate": "2015-04-20T11:04:00.000+10:00",
        "originBoardId": 5
    }"#;

    let sprint: Sprint = serde_json::from_str(sprint_str).unwrap();

    assert_eq!(sprint.id, 72u64);
    assert_eq!(sprint.state, Some("future".to_owned()));
    assert_eq!(sprint.name, "sprint 2");
    assert_eq!(
        sprint.self_link,
        "http://www.example.com/jira/rest/agile/1.0/sprint/73"
    );
    assert_eq!(
        sprint.start_date,
        Some(datetime!(2015-04-11 15:22:00.000 +10:00))
    );

    assert_eq!(
        sprint.end_date,
        Some(datetime!(2015-04-20 01:22:00.000 +10:00))
    );
    assert_eq!(
        sprint.complete_date,
        Some(datetime!(2015-04-20 11:04:00.000 +10:00))
    );
    assert_eq!(sprint.origin_board_id, Some(5));
}

#[test]
fn deserialise_sprint_results() {
    let sprint_results_str = r#"{
        "maxResults": 50,
        "startAt": 0,
        "isLast": true,
        "values": [{
            "id": 72,
            "self": "http://www.example.com/jira/rest/agile/1.0/sprint/73",
            "state": "future",
            "name": "sprint 2"
        }]
    }"#;

    let sprint_results: SprintResults = serde_json::from_str(sprint_results_str).unwrap();

    assert_eq!(sprint_results.max_results, 50u64);
    assert_eq!(sprint_results.start_at, 0u64);
    assert!(sprint_results.is_last);
    assert_eq!(sprint_results.values.len(), 1);
}

#[test]
fn test_update_sprint_date_serialization() {
    use time::OffsetDateTime;

    // Test UTC timezone - should produce format: "2024-01-01T09:00:00.000+0000"
    let start = OffsetDateTime::from_unix_timestamp(1704096000).unwrap(); // 2024-01-01 08:00:00 UTC
    let end = OffsetDateTime::from_unix_timestamp(1704182400).unwrap(); // 2024-01-02 08:00:00 UTC

    let update = UpdateSprint {
        name: Some("Sprint 1".to_string()),
        start_date: Some(start),
        end_date: Some(end),
        state: Some("active".to_string()),
    };

    let serialized = serde_json::to_string(&update).unwrap();
    println!("UpdateSprint serialized: {}", serialized);

    // Verify the format matches JIRA expectations
    assert!(serialized.contains("\"startDate\":\"2024-01-01T08:00:00.000+0000\""));
    assert!(serialized.contains("\"endDate\":\"2024-01-02T08:00:00.000+0000\""));
    assert!(!serialized.contains("+002024")); // Should not have year prefix
    assert!(!serialized.contains("Z\"")); // Should not use Z notation
    assert!(serialized.contains("\"name\":\"Sprint 1\""));
    assert!(serialized.contains("\"state\":\"active\""));
}

#[test]
fn test_update_sprint_date_with_timezone() {
    use time::{OffsetDateTime, UtcOffset};

    // Test with +10:00 timezone (Australian Eastern)
    let utc_time = OffsetDateTime::from_unix_timestamp(1704096000).unwrap();
    let aest_offset = UtcOffset::from_hms(10, 0, 0).unwrap();
    let aest_time = utc_time.to_offset(aest_offset);

    let update = UpdateSprint {
        name: None,
        start_date: Some(aest_time),
        end_date: None,
        state: None,
    };

    let serialized = serde_json::to_string(&update).unwrap();
    println!("UpdateSprint with AEST: {}", serialized);

    // Should include +1000 offset and proper date in that timezone
    assert!(serialized.contains("+1000"));
    assert!(serialized.contains("2024-01-01T18:00:00.000+1000"));
}

#[test]
fn test_update_sprint_without_dates() {
    // Test that omitting date fields works correctly
    let update = UpdateSprint {
        name: Some("Sprint 2".to_string()),
        start_date: None,
        end_date: None,
        state: Some("future".to_string()),
    };

    let serialized = serde_json::to_string(&update).unwrap();
    println!("UpdateSprint without dates: {}", serialized);

    // Should not include date fields at all
    assert!(!serialized.contains("\"startDate\""));
    assert!(!serialized.contains("\"endDate\""));
    assert!(serialized.contains("\"name\":\"Sprint 2\""));
    assert!(serialized.contains("\"state\":\"future\""));
}
