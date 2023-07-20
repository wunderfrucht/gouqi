use crate::{Jira, Result, Version, VersionCreationBody, VersionMoveAfterBody, VersionUpdateBody};

pub struct Versions {
    jira: Jira,
}

impl Versions {
    /// Creates a new Versions struct that interacts with the provided Jira client.
    ///
    /// # Arguments
    ///
    /// * `jira` - Reference to a Jira client instance.
    ///
    /// # Returns
    ///
    /// Returns a new Versions instance.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::Jira;
    ///
    /// let jira = Jira::new("https://my.jira.com");
    /// let versions = Versions::new(&jira);
    /// ```
    pub fn new(jira: &Jira) -> Self {
        Self { jira: jira.clone() }
    }

    /// Fetches all versions associated with the given project ID or key.
    ///
    /// # Arguments
    ///
    /// * `project_id_or_key` - Project ID or key to fetch versions for.  
    ///
    /// # Returns
    ///
    /// Returns a Result containing a vector of Version structs.
    ///
    /// # Example
    ///
    /// ```
    /// let versions = versions.project_versions("PROJ");
    /// ```
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-project-projectIdOrKey-versions-get)
    /// for more information
    pub fn project_versions(&self, project_id_or_key: &str) -> Result<Vec<Version>> {
        self.jira
            .get("api", &format!("/project/{project_id_or_key}/versions"))
    }

    /// Creates a new version for the given project ID and name.
    ///
    /// # Arguments
    ///
    /// * `project_id` - ID of the project to create the version for.
    /// * `name` - Name of the new version.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created Version struct.
    ///
    /// # Example
    ///
    /// ```
    /// versions.create(123, "1.0");
    /// ```
    ///
    /// See [jira docs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/#api-rest-api-2-version-post)
    /// for more information
    pub fn create<T: Into<String>>(&self, project_id: u64, name: T) -> Result<Version> {
        let name = name.into();
        self.jira
            .post("api", "/version", VersionCreationBody { project_id, name })
    }

    /// Moves the given version after the specified version.
    ///
    /// # Arguments
    ///
    /// * `version` - Version struct to move.
    /// * `after` - Self link of the version to move after.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the moved Version struct.
    ///
    /// # Example
    ///
    /// ```
    /// let v1 = // existing version
    /// let v2 = // existing version
    /// versions.move_after(&v1, v2.self_link);
    /// ```
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

    /// Releases the given version by setting released to true.
    ///
    /// # Arguments
    ///
    /// * `version` - Version struct to release.
    /// * `move_unfixed_issues_to` - Optional version to move unfixed issues to.
    ///
    /// # Returns
    ///  
    /// Returns a Result with no value if successful.
    ///
    /// # Example
    ///
    /// ```
    /// let v1 = // existing unreleased version
    /// versions.release(&v1, None);
    /// ```
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
}
