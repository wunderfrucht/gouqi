//! OAuth 1.0a authentication implementation for Jira Server/Data Center
//!
//! This module implements OAuth 1.0a with RSA-SHA1 request signing as specified by
//! Atlassian for Jira Server and Data Center deployments.
//!
//! # OAuth 1.0a Flow
//!
//! 1. **Request Token**: Get a request token from Jira
//! 2. **User Authorization**: Direct user to authorize the application
//! 3. **Access Token**: Exchange authorized request token for an access token
//! 4. **Signed Requests**: Sign all API requests with the access token
//!
//! # Usage
//!
//! This module provides the signing functionality for step 4. The authorization flow
//! (steps 1-3) must be implemented separately, typically in your application's web server.
//!
//! Once you have obtained OAuth credentials through the authorization flow, use them
//! with the `Credentials::OAuth1a` variant.
//!
//! # Security Note
//!
//! This module uses `rsa 0.9.8` which has a known timing sidechannel vulnerability
//! ([RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071)) in PKCS#1 v1.5
//! **decryption**. This vulnerability does NOT affect our implementation because we only
//! use RSA for **signing**, not decryption. The vulnerability will be resolved when we
//! upgrade to `rsa 0.10.0` once it's stable.
//!
//! # References
//!
//! - [Jira OAuth Documentation](https://developer.atlassian.com/server/jira/platform/oauth/)
//! - [RFC 5849 - OAuth 1.0 Protocol](https://tools.ietf.org/html/rfc5849)

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, distributions::Alphanumeric, thread_rng};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    signature::{SignatureEncoding, Signer},
};
use sha1::Sha1;

use crate::{Error, Result};

/// Generate OAuth 1.0a authorization header for a request
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `url` - Full request URL
/// * `consumer_key` - OAuth consumer key
/// * `private_key_pem` - RSA private key in PEM format
/// * `token` - OAuth access token
/// * `token_secret` - OAuth access token secret (currently unused for RSA-SHA1, but kept for API compatibility)
///
/// # Returns
///
/// The complete OAuth authorization header value (including "OAuth " prefix)
///
/// # Errors
///
/// Returns an error if:
/// - RSA private key cannot be decoded
/// - Signature generation fails
///
/// # Panics
///
/// This function will panic if system time is before UNIX epoch
pub fn generate_oauth_header(
    method: &str,
    url: &str,
    consumer_key: &str,
    private_key_pem: &str,
    token: &str,
    _token_secret: &str, // Not used for RSA-SHA1
) -> Result<String> {
    // Generate timestamp and nonce
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        .to_string();

    let nonce = generate_nonce();

    // Build OAuth parameters
    let mut oauth_params = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key", consumer_key.to_string());
    oauth_params.insert("oauth_nonce", nonce);
    oauth_params.insert("oauth_signature_method", "RSA-SHA1".to_string());
    oauth_params.insert("oauth_timestamp", timestamp);
    oauth_params.insert("oauth_token", token.to_string());
    oauth_params.insert("oauth_version", "1.0".to_string());

    // Parse URL to extract base URL and query parameters
    let (base_url, query_params) = parse_url(url);

    // Generate signature base string
    let signature_base = build_signature_base(method, &base_url, &oauth_params, &query_params);

    // Generate RSA-SHA1 signature
    let signature = sign_rsa_sha1(private_key_pem, &signature_base)?;

    // Add signature to OAuth parameters
    oauth_params.insert("oauth_signature", signature);

    // Build authorization header
    let auth_header = oauth_params
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, percent_encode(v)))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!("OAuth {}", auth_header))
}

/// Generate a cryptographically secure nonce
///
/// # Returns
///
/// A 32-character random alphanumeric string
fn generate_nonce() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Parse URL into base URL and query parameters
///
/// # Arguments
///
/// * `url` - The full URL to parse
///
/// # Returns
///
/// A tuple of (base_url, query_parameters)
fn parse_url(url: &str) -> (String, BTreeMap<String, String>) {
    let mut query_params = BTreeMap::new();

    if let Some(question_mark) = url.find('?') {
        let base_url = url[..question_mark].to_string();
        let query_string = &url[question_mark + 1..];

        for pair in query_string.split('&') {
            if let Some(equals) = pair.find('=') {
                let key = &pair[..equals];
                let value = &pair[equals + 1..];
                query_params.insert(key.to_string(), value.to_string());
            }
        }

        (base_url, query_params)
    } else {
        (url.to_string(), query_params)
    }
}

/// Build OAuth signature base string according to RFC 5849
///
/// # Arguments
///
/// * `method` - HTTP method (uppercase)
/// * `base_url` - Base URL without query parameters
/// * `oauth_params` - OAuth protocol parameters
/// * `query_params` - URL query parameters
///
/// # Returns
///
/// The signature base string
fn build_signature_base(
    method: &str,
    base_url: &str,
    oauth_params: &BTreeMap<&str, String>,
    query_params: &BTreeMap<String, String>,
) -> String {
    // Combine OAuth and query parameters
    let mut all_params = BTreeMap::new();

    for (k, v) in oauth_params {
        // Skip oauth_signature if present (it shouldn't be at this point, but just in case)
        if *k != "oauth_signature" {
            all_params.insert(k.to_string(), v.clone());
        }
    }

    for (k, v) in query_params {
        all_params.insert(k.clone(), v.clone());
    }

    // Sort parameters and create normalized string
    let normalized_params = all_params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Build signature base string: METHOD&URL&PARAMS
    format!(
        "{}&{}&{}",
        method.to_uppercase(),
        percent_encode(base_url),
        percent_encode(&normalized_params)
    )
}

/// Sign data using RSA-SHA1
///
/// # Arguments
///
/// * `private_key_pem` - RSA private key in PEM format
/// * `data` - Data to sign
///
/// # Returns
///
/// Base64-encoded signature
///
/// # Errors
///
/// Returns an error if:
/// - Private key cannot be decoded
/// - Signing fails
fn sign_rsa_sha1(private_key_pem: &str, data: &str) -> Result<String> {
    // Parse RSA private key
    let private_key =
        RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|e| Error::OAuthError {
            message: format!("Failed to parse RSA private key: {}", e),
        })?;

    // Create signing key (RSA requires unprefixed for SHA1)
    let signing_key = SigningKey::<Sha1>::new_unprefixed(private_key);

    // Sign the data
    let signature = signing_key
        .try_sign(data.as_bytes())
        .map_err(|e| Error::OAuthError {
            message: format!("Failed to sign data: {}", e),
        })?;

    // Return base64-encoded signature
    Ok(general_purpose::STANDARD.encode(signature.to_bytes()))
}

/// Percent-encode a string according to RFC 3986
///
/// # Arguments
///
/// * `input` - String to encode
///
/// # Returns
///
/// Percent-encoded string
fn percent_encode(input: &str) -> String {
    input
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{:02X}", byte),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for test
    fn percent_decode(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            } else {
                result.push(ch);
            }
        }
        result
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("a-b.c_d~e"), "a-b.c_d~e");
        assert_eq!(percent_encode("special!@#"), "special%21%40%23");
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        assert_eq!(nonce1.len(), 32);
        assert_eq!(nonce2.len(), 32);
        assert_ne!(nonce1, nonce2); // Should be random
    }

    #[test]
    fn test_parse_url() {
        let (base, params) = parse_url("https://example.com/api/search?q=test&limit=10");
        assert_eq!(base, "https://example.com/api/search");
        assert_eq!(params.get("q"), Some(&"test".to_string()));
        assert_eq!(params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_parse_url_no_query() {
        let (base, params) = parse_url("https://example.com/api/search");
        assert_eq!(base, "https://example.com/api/search");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_signature_base() {
        let method = "GET";
        let base_url = "https://example.com/api/search";
        let mut oauth_params = BTreeMap::new();
        oauth_params.insert("oauth_consumer_key", "key".to_string());
        oauth_params.insert("oauth_nonce", "nonce".to_string());

        let mut query_params = BTreeMap::new();
        query_params.insert("q".to_string(), "test".to_string());

        let base = build_signature_base(method, base_url, &oauth_params, &query_params);

        assert!(base.starts_with("GET&"));
        assert!(base.contains("https%3A%2F%2Fexample.com%2Fapi%2Fsearch"));
        assert!(base.contains("oauth_consumer_key"));
        assert!(base.contains("q%3Dtest"));
    }

    #[test]
    fn test_parse_url_with_multiple_params() {
        let (base, params) = parse_url(
            "https://jira.example.com/rest/api/2/search?jql=project=TEST&maxResults=50&startAt=0",
        );
        assert_eq!(base, "https://jira.example.com/rest/api/2/search");
        assert_eq!(params.get("jql"), Some(&"project=TEST".to_string()));
        assert_eq!(params.get("maxResults"), Some(&"50".to_string()));
        assert_eq!(params.get("startAt"), Some(&"0".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_percent_encode_unreserved_chars() {
        // RFC 3986 unreserved characters should not be encoded
        assert_eq!(
            percent_encode("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"
        );
    }

    #[test]
    fn test_percent_encode_reserved_chars() {
        // Reserved characters should be encoded
        assert_eq!(
            percent_encode(":/?#[]@!$&'()*+,;="),
            "%3A%2F%3F%23%5B%5D%40%21%24%26%27%28%29%2A%2B%2C%3B%3D"
        );
    }

    #[test]
    fn test_sign_rsa_sha1_invalid_key() {
        let invalid_key = "-----BEGIN PRIVATE KEY-----\nINVALID\n-----END PRIVATE KEY-----";
        let result = sign_rsa_sha1(invalid_key, "test data");
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_header_invalid_key_returns_error() {
        let invalid_key = "-----BEGIN PRIVATE KEY-----\nNOT_A_VALID_KEY\n-----END PRIVATE KEY-----";

        let result = generate_oauth_header(
            "GET",
            "https://jira.example.com/rest/api/2/myself",
            "consumer-key",
            invalid_key,
            "access-token",
            "access-token-secret",
        );

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("OAuth"));
        }
    }

    #[test]
    fn test_build_signature_base_excludes_oauth_signature() {
        let method = "POST";
        let base_url = "https://jira.example.com/rest/api/2/issue";
        let mut oauth_params = BTreeMap::new();
        oauth_params.insert("oauth_consumer_key", "key".to_string());
        oauth_params.insert("oauth_signature", "should-be-excluded".to_string());
        oauth_params.insert("oauth_nonce", "nonce".to_string());

        let query_params = BTreeMap::new();

        let base = build_signature_base(method, base_url, &oauth_params, &query_params);

        // oauth_signature should not be included in the base string
        assert!(!base.contains("should-be-excluded"));
        assert!(!base.contains("oauth_signature"));
    }

    #[test]
    fn test_build_signature_base_parameter_ordering() {
        let method = "GET";
        let base_url = "https://example.com/api";
        let mut oauth_params = BTreeMap::new();
        // Add parameters in non-alphabetical order
        oauth_params.insert("oauth_version", "1.0".to_string());
        oauth_params.insert("oauth_consumer_key", "key".to_string());
        oauth_params.insert("oauth_nonce", "nonce".to_string());

        let query_params = BTreeMap::new();

        let base = build_signature_base(method, base_url, &oauth_params, &query_params);

        // Verify parameters are sorted alphabetically
        let params_part = base.split('&').nth(2).unwrap();
        let decoded = percent_decode(params_part);

        // BTreeMap maintains sorted order, so oauth_consumer_key should come before oauth_nonce
        let consumer_pos = decoded.find("oauth_consumer_key").unwrap();
        let nonce_pos = decoded.find("oauth_nonce").unwrap();
        let version_pos = decoded.find("oauth_version").unwrap();

        assert!(consumer_pos < nonce_pos);
        assert!(nonce_pos < version_pos);
    }
}
