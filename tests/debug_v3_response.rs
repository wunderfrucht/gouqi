//! Debug test to inspect the actual V3 API response
//! This helps us understand what format the V3 API returns

use gouqi::{Credentials, Jira, SearchApiVersion};

#[test]
fn debug_v3_raw_response() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping debug test");
            return;
        }
    };

    if token.trim().is_empty() {
        eprintln!("⚠️  INTEGRATION_JIRA_TOKEN is empty, skipping debug test");
        return;
    }

    println!("🔍 Making raw HTTP request to V3 API to inspect response...");

    // Create a simple HTTP client to make the request directly
    let client = reqwest::blocking::Client::new();

    let url = "https://gouji.atlassian.net/rest/api/3/search/jql?maxResults=1";

    println!("🌐 Making request to: {}", url);

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send();

    match response {
        Ok(resp) => {
            let status = resp.status();
            println!("📊 Status Code: {}", status);

            match resp.text() {
                Ok(body) => {
                    println!("📄 Response Body:");
                    println!("{}", body);

                    if status.is_client_error() {
                        println!("❌ This is a 4xx error - let's see the error format");

                        // Try parsing as our expected Errors format
                        match serde_json::from_str::<gouqi::Errors>(&body) {
                            Ok(errors) => {
                                println!("✅ Successfully parsed as gouqi::Errors format");
                                println!("   Error messages: {:?}", errors.error_messages);
                                println!("   Errors map: {:?}", errors.errors);
                            }
                            Err(e) => {
                                println!("❌ Failed to parse as gouqi::Errors: {}", e);
                                println!(
                                    "   This suggests V3 API returns errors in different format"
                                );
                            }
                        }
                    } else if status.is_success() {
                        println!("✅ This is a success response - let's see the format");

                        // Try parsing as SearchResults
                        match serde_json::from_str::<gouqi::SearchResults>(&body) {
                            Ok(results) => {
                                println!("✅ Successfully parsed as SearchResults");
                                println!("   Total: {}", results.total);
                                println!("   Issues found: {}", results.issues.len());
                            }
                            Err(e) => {
                                println!("❌ Failed to parse as SearchResults: {}", e);
                                println!(
                                    "   This suggests V3 API returns search results in different format"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to read response body: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
        }
    }
}

#[test]
fn debug_v2_vs_v3_comparison() {
    let token = match std::env::var("INTEGRATION_JIRA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️  INTEGRATION_JIRA_TOKEN not set, skipping comparison test");
            return;
        }
    };

    if token.trim().is_empty() {
        return;
    }

    println!("🔍 Comparing V2 vs V3 API responses...");

    let credentials = Credentials::Bearer(token);

    // Test V2 API
    println!("📡 Testing V2 API response...");
    let jira_v2 = Jira::with_search_api_version(
        "https://gouji.atlassian.net",
        credentials.clone(),
        SearchApiVersion::V2,
    )
    .expect("Failed to create V2 client");

    let search_options = gouqi::SearchOptions::builder().max_results(1).build();

    match jira_v2.search().list("", &search_options) {
        Ok(results) => {
            println!("✅ V2 API successful - found {} issues", results.total);
        }
        Err(e) => {
            println!("❌ V2 API failed: {:?}", e);
        }
    }

    // Test V3 API
    println!("📡 Testing V3 API response...");
    let jira_v3 = Jira::with_search_api_version(
        "https://gouji.atlassian.net",
        credentials,
        SearchApiVersion::V3,
    )
    .expect("Failed to create V3 client");

    match jira_v3.search().list("", &search_options) {
        Ok(results) => {
            println!("✅ V3 API successful - found {} issues", results.total);
        }
        Err(e) => {
            println!("❌ V3 API failed: {:?}", e);
        }
    }
}
