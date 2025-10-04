use crate::{Jira, Result, Version, VersionCreationBody, VersionMoveAfterBody, VersionUpdateBody};

pub struct Versions {
    jira: Jira,
}

impl Versions {
    pub fn new(jira: &Jira) -> Self {
        Self { jira: jira.clone() }
    }

    /// Fetch all versions associated to the given project
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-project-projectIdOrKey-versions-get)
    /// for more information
    pub fn project_versions(&self, project_id_or_key: &str) -> Result<Vec<Version>> {
        self.jira
            .get("api", &format!("/project/{project_id_or_key}/versions"))
    }

    /// Create a new version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-post)
    /// for more information
    pub fn create<T: Into<String>>(&self, project_id: u64, name: T) -> Result<Version> {
        let name = name.into();
        self.jira
            .post("api", "/version", VersionCreationBody { project_id, name })
    }

    /// Move a version after another version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-id-move-post)
    /// for more information
    pub fn move_after<T: Into<String>>(&self, version: &Version, after: T) -> Result<Version> {
        self.jira.post(
            "api",
            &format!("/version/{}/move", version.id),
            VersionMoveAfterBody {
                after: after.into(),
            },
        )
    }

    /// Release a new version: modify the version by turning the released boolean to true
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-id-put)
    /// for more information
    pub fn release(
        &self,
        version: &Version,
        move_unfixed_issues_to: Option<&Version>,
    ) -> Result<()> {
        if version.released {
            // already released
            Ok(())
        } else {
            self.jira
                .put::<Version, _>(
                    "api",
                    &format!("/version/{}", version.id),
                    VersionUpdateBody {
                        released: true,
                        archived: false,
                        move_unfixed_issues_to: move_unfixed_issues_to.map(|v| v.self_link.clone()),
                    },
                )
                .map(|_v| ())
        }
    }

    /// Delete a version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-versions/#api-rest-api-2-version-id-delete)
    /// for more information
    pub fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/version/{}", id.into()))?;
        Ok(())
    }
}

#[cfg(feature = "async")]
use crate::r#async::Jira as AsyncJira;

#[cfg(feature = "async")]
pub struct AsyncVersions {
    jira: AsyncJira,
}

#[cfg(feature = "async")]
impl AsyncVersions {
    pub fn new(jira: &AsyncJira) -> Self {
        Self { jira: jira.clone() }
    }

    /// Fetch all versions associated to the given project
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-project-projectIdOrKey-versions-get)
    /// for more information
    pub async fn project_versions(&self, project_id_or_key: &str) -> Result<Vec<Version>> {
        self.jira
            .get("api", &format!("/project/{project_id_or_key}/versions"))
            .await
    }

    /// Create a new version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-post)
    /// for more information
    pub async fn create<T: Into<String>>(&self, project_id: u64, name: T) -> Result<Version> {
        let name = name.into();
        self.jira
            .post("api", "/version", VersionCreationBody { project_id, name })
            .await
    }

    /// Move a version after another version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-id-move-post)
    /// for more information
    pub async fn move_after<T: Into<String>>(
        &self,
        version: &Version,
        after: T,
    ) -> Result<Version> {
        self.jira
            .post(
                "api",
                &format!("/version/{}/move", version.id),
                VersionMoveAfterBody {
                    after: after.into(),
                },
            )
            .await
    }

    /// Release a new version: modify the version by turning the released boolean to true
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-id-put)
    /// for more information
    pub async fn release(
        &self,
        version: &Version,
        move_unfixed_issues_to: Option<&Version>,
    ) -> Result<()> {
        if version.released {
            // already released
            Ok(())
        } else {
            self.jira
                .put::<Version, _>(
                    "api",
                    &format!("/version/{}", version.id),
                    VersionUpdateBody {
                        released: true,
                        archived: false,
                        move_unfixed_issues_to: move_unfixed_issues_to.map(|v| v.self_link.clone()),
                    },
                )
                .await
                .map(|_v| ())
        }
    }

    /// Delete a version
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-versions/#api-rest-api-2-version-id-delete)
    /// for more information
    pub async fn delete<I>(&self, id: I) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira
            .delete::<crate::EmptyResponse>("api", &format!("/version/{}", id.into()))
            .await?;
        Ok(())
    }
}
