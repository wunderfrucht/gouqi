//! Interfaces for accessing and managing attachments

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Ours
use crate::{Jira, Result};

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

/// Same as `User`, but without `email_address`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    pub active: bool,
    #[serde(rename = "avatarUrls")]
    pub avatar_urls: BTreeMap<String, String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub key: Option<String>,
    pub name: String,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "timeZone")]
    pub timezone: Option<String>,
}

/// Same as `Attachement`, but without `id` and with `UserResponse`
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentResponse {
    #[serde(rename = "self")]
    pub self_link: String,
    pub filename: String,
    pub author: UserResponse,
    pub created: String,
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub content: String,
    pub thumbnail: Option<String>,
}

#[derive(Debug)]
pub struct Attachments {
    jira: Jira,
}

impl Attachments {
    pub fn new(jira: &Jira) -> Attachments {
        Attachments { jira: jira.clone() }
    }

    /// Get the meta-data of a single attachment
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/attachment-getAttachment)
    /// for more information
    pub fn get<I>(&self, id: I) -> Result<AttachmentResponse>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/attachment/{}", id.into()))
    }

    /// Delete a single attachment
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/attachment-removeAttachment)
    /// for more information
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/attachment/{}", id.into()))?;
        Ok(())
    }

    /// Download attachment content as raw bytes
    ///
    /// This method retrieves the actual file content of an attachment. It automatically
    /// handles authentication using the same credentials as other API calls.
    ///
    /// # Arguments
    ///
    /// * `id` - The attachment ID
    ///
    /// # Returns
    ///
    /// `Result<Vec<u8>>` - The raw attachment content as bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gouqi::{Credentials, Jira};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let jira = Jira::new("https://jira.example.com", Credentials::Anonymous)?;
    /// let content_bytes = jira.attachments().download("12345")?;
    ///
    /// // Save to file
    /// std::fs::write("attachment.pdf", &content_bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See this [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-attachments/#api-rest-api-3-attachment-content-id-get)
    /// for more information
    pub fn download<I>(&self, id: I) -> Result<Vec<u8>>
    where
        I: Into<String>,
    {
        // The attachment content endpoint returns raw bytes, not JSON
        self.jira
            .get_bytes("api", &format!("/attachment/content/{}", id.into()))
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncAttachments {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncAttachments {
    pub fn new(jira: &AsyncJira) -> AsyncAttachments {
        AsyncAttachments { jira: jira.clone() }
    }

    /// Get the meta-data of a single attachment
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/attachment-getAttachment)
    /// for more information
    pub async fn get<I>(&self, id: I) -> Result<AttachmentResponse>
    where
        I: Into<String>,
    {
        self.jira
            .get("api", &format!("/attachment/{}", id.into()))
            .await
    }

    /// Delete a single attachment
    ///
    /// See this [jira docs](https://docs.atlassian.com/software/jira/docs/api/REST/8.13.8/#api/2/attachment-removeAttachment)
    /// for more information
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/attachment/{}", id.into()))
            .await?;
        Ok(())
    }

    /// Download attachment content as raw bytes (async)
    ///
    /// This method retrieves the actual file content of an attachment. It automatically
    /// handles authentication using the same credentials as other API calls.
    ///
    /// # Arguments
    ///
    /// * `id` - The attachment ID
    ///
    /// # Returns
    ///
    /// `Result<Vec<u8>>` - The raw attachment content as bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gouqi::{Credentials, r#async::Jira};
    ///
    /// let jira = Jira::new("https://jira.example.com", Credentials::Anonymous)?;
    /// let content_bytes = jira.attachments().download("12345").await?;
    ///
    /// // Save to file
    /// std::fs::write("attachment.pdf", &content_bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See this [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-attachments/#api-rest-api-3-attachment-content-id-get)
    /// for more information
    pub async fn download<I>(&self, id: I) -> Result<Vec<u8>>
    where
        I: Into<String>,
    {
        // The attachment content endpoint returns raw bytes, not JSON
        self.jira
            .get_bytes("api", &format!("/attachment/content/{}", id.into()))
            .await
    }
}
