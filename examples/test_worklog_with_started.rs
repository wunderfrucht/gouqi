//! Example demonstrating worklog creation with ergonomic time helpers
//! This tests the fix for issue #123 and showcases new helper methods

use gouqi::WorklogInput;
use time::{OffsetDateTime, macros::datetime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing worklog with ergonomic time helpers...\n");

    // Example 1: Using time unit helpers
    println!("=== Example 1: Creating worklogs with different time units ===");
    let from_hours = WorklogInput::from_hours(2);
    let from_minutes = WorklogInput::from_minutes(30);
    let from_days = WorklogInput::from_days(1); // 8 hour workday
    let from_weeks = WorklogInput::from_weeks(1); // 5 day workweek

    println!(
        "2 hours = {} seconds",
        from_hours.time_spent_seconds.unwrap()
    );
    println!(
        "30 minutes = {} seconds",
        from_minutes.time_spent_seconds.unwrap()
    );
    println!(
        "1 day = {} seconds (8h workday)",
        from_days.time_spent_seconds.unwrap()
    );
    println!(
        "1 week = {} seconds (5d workweek)",
        from_weeks.time_spent_seconds.unwrap()
    );

    // Example 2: Using relative time helpers
    println!("\n=== Example 2: Setting started time relative to now ===");

    // These create worklogs with start times relative to now
    WorklogInput::from_hours(2)
        .started_hours_ago(3)
        .with_comment("Started 3 hours ago");
    println!("✓ Can create worklog that started 3 hours ago");

    WorklogInput::from_hours(4)
        .started_days_ago(2)
        .with_comment("Started 2 days ago");
    println!("✓ Can create worklog that started 2 days ago");

    // Example 3: Using specific datetime
    println!("\n=== Example 3: Setting specific start time ===");
    let specific_time = datetime!(2024-01-15 09:00:00 UTC);
    WorklogInput::from_hours(3)
        .started_at(specific_time)
        .with_comment("Started at specific time");
    println!("✓ Worklog started at: {}", specific_time);

    // Example 4: Complete workflow example with correct date formatting
    println!("\n=== Example 4: Complete worklog with JIRA-compatible format ===");
    let started = OffsetDateTime::from_unix_timestamp(1704096000)?;

    let worklog = WorklogInput::from_hours(2)
        .with_comment("Fixed critical bug - issue #123 fix validated")
        .with_started(started);

    // Serialize to see the format
    let serialized = serde_json::to_string_pretty(&worklog)?;
    println!("\nWorklog JSON (should have correct JIRA format):");
    println!("{}", serialized);

    // Verify the format
    let serialized_compact = serde_json::to_string(&worklog)?;
    assert!(
        serialized_compact.contains("2024-01-01T08:00:00.000+0000"),
        "Date format incorrect"
    );
    assert!(
        !serialized_compact.contains("+002024"),
        "Should not have year prefix"
    );
    assert!(
        !serialized_compact.contains("Z\""),
        "Should not use Z notation"
    );

    println!("\n✓ Date format validation passed!");
    println!("✓ Format matches JIRA expectations: 2024-01-01T08:00:00.000+0000");

    // Example 5: Practical real-world usage
    println!("\n=== Example 5: Real-world usage patterns ===");

    // Log work that happened yesterday
    WorklogInput::from_hours(6)
        .started_days_ago(1)
        .with_comment("Implemented user authentication");
    println!("✓ Created worklog for yesterday");

    // Log work from this morning
    WorklogInput::from_minutes(45)
        .started_hours_ago(4)
        .with_comment("Code review and bug fixes");
    println!("✓ Created worklog from this morning");

    // Log work for a specific date/time
    WorklogInput::from_hours(3)
        .started_at(datetime!(2024-01-10 14:00:00 UTC))
        .with_comment("Backend API development");
    println!("✓ Created worklog for specific date");

    println!("\n=== Summary ===");
    println!("✓ Issue #123 fixed - WorklogInput now uses correct JIRA date format");
    println!("✓ Added ergonomic time helpers: from_hours, from_minutes, from_days, from_weeks");
    println!("✓ Added relative time helpers: started_hours_ago, started_days_ago, etc.");
    println!("✓ All datetime values are correctly serialized for JIRA API");

    Ok(())
}
