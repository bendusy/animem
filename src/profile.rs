use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalProfile {
    #[serde(default)]
    pub schema_version: Option<String>,
    pub name: String,
    #[serde(default)]
    pub sources: Vec<DataSource>,
    #[serde(default)]
    pub maintenance: MaintenancePolicy,
    #[serde(default)]
    pub path_privacy: PathPrivacy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSource {
    pub source_id: String,
    pub label: String,
    pub root: String,
    #[serde(default)]
    pub include_extensions: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenancePolicy {
    pub batch_prefix: String,
    pub require_dry_run: bool,
    pub write_source_paths: bool,
}

impl Default for MaintenancePolicy {
    fn default() -> Self {
        Self {
            batch_prefix: "manual".to_string(),
            require_dry_run: true,
            write_source_paths: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathPrivacy {
    StoreFullPath,
    #[default]
    StoreRelativePath,
    StoreLabelOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileValidationError {
    pub field: &'static str,
    pub message: String,
}

impl fmt::Display for ProfileValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ProfileValidationError {}

impl LocalProfile {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_optional_text("schema_version", self.schema_version.as_deref())?;
        validate_non_empty("name", &self.name)?;
        if self.sources.is_empty() {
            return Err(ProfileValidationError {
                field: "sources",
                message: "at least one source is required".to_string(),
            });
        }
        for source in &self.sources {
            source.validate()?;
        }
        validate_non_empty("maintenance.batch_prefix", &self.maintenance.batch_prefix)?;
        Ok(())
    }

    pub fn source_ref(
        &self,
        source_id: &str,
        path: impl AsRef<Path>,
    ) -> std::result::Result<String, ProfileValidationError> {
        let source = self
            .sources
            .iter()
            .find(|source| source.source_id == source_id)
            .ok_or_else(|| ProfileValidationError {
                field: "sources.source_id",
                message: format!("unknown source_id '{}'", source_id),
            })?;
        source.source_ref(path, self.path_privacy)
    }
}

impl DataSource {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_token("sources.source_id", &self.source_id)?;
        validate_non_empty("sources.label", &self.label)?;
        validate_non_empty("sources.root", &self.root)?;
        for ext in &self.include_extensions {
            validate_token("sources.include_extensions", ext)?;
        }
        Ok(())
    }

    pub fn source_ref(
        &self,
        path: impl AsRef<Path>,
        privacy: PathPrivacy,
    ) -> std::result::Result<String, ProfileValidationError> {
        let path = path.as_ref();
        match privacy {
            PathPrivacy::StoreFullPath => Ok(path.to_string_lossy().to_string()),
            PathPrivacy::StoreLabelOnly => {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .ok_or_else(|| ProfileValidationError {
                        field: "path",
                        message: "path has no file name".to_string(),
                    })?;
                Ok(format!("{}:{}", self.source_id, name))
            }
            PathPrivacy::StoreRelativePath => {
                let rel = path
                    .strip_prefix(&self.root)
                    .map_err(|_| ProfileValidationError {
                        field: "path",
                        message: "path is outside configured source root".to_string(),
                    })?;
                Ok(format!(
                    "{}:{}",
                    self.source_id,
                    rel.to_string_lossy().trim_start_matches('/')
                ))
            }
        }
    }
}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> std::result::Result<(), ProfileValidationError> {
    if value.trim().is_empty() {
        return Err(ProfileValidationError {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
) -> std::result::Result<(), ProfileValidationError> {
    if let Some(value) = value {
        validate_non_empty(field, value)?;
    }
    Ok(())
}

fn validate_token(
    field: &'static str,
    value: &str,
) -> std::result::Result<(), ProfileValidationError> {
    validate_non_empty(field, value)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(ProfileValidationError {
            field,
            message: "must use only ASCII letters, numbers, '-', '_' or '.'".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_synthetic_profile() {
        let profile = LocalProfile {
            schema_version: Some("1".to_string()),
            name: "example".to_string(),
            sources: vec![DataSource {
                source_id: "policy-memos".to_string(),
                label: "Policy Memo Samples".to_string(),
                root: "/example/private/source".to_string(),
                include_extensions: vec!["md".to_string(), "txt".to_string()],
                exclude_globs: vec!["**/draft/**".to_string()],
            }],
            maintenance: MaintenancePolicy {
                batch_prefix: "example-batch".to_string(),
                require_dry_run: true,
                write_source_paths: false,
            },
            path_privacy: PathPrivacy::StoreRelativePath,
        };

        profile.validate().unwrap();
    }

    #[test]
    fn rejects_missing_sources() {
        let profile = LocalProfile {
            schema_version: None,
            name: "example".to_string(),
            sources: vec![],
            maintenance: MaintenancePolicy::default(),
            path_privacy: PathPrivacy::default(),
        };

        let err = profile.validate().unwrap_err();
        assert_eq!(err.field, "sources");
    }

    #[test]
    fn builds_relative_source_reference() {
        let profile = LocalProfile {
            schema_version: Some("1".to_string()),
            name: "example".to_string(),
            sources: vec![DataSource {
                source_id: "policy-memos".to_string(),
                label: "Policy Memo Samples".to_string(),
                root: "/example/private/source".to_string(),
                include_extensions: vec!["md".to_string()],
                exclude_globs: vec![],
            }],
            maintenance: MaintenancePolicy::default(),
            path_privacy: PathPrivacy::StoreRelativePath,
        };

        let reference = profile
            .source_ref("policy-memos", "/example/private/source/a/b.md")
            .unwrap();
        assert_eq!(reference, "policy-memos:a/b.md");
    }

    #[test]
    fn rejects_relative_reference_outside_root() {
        let source = DataSource {
            source_id: "notes".to_string(),
            label: "Notes".to_string(),
            root: "/example/private/source".to_string(),
            include_extensions: vec!["md".to_string()],
            exclude_globs: vec![],
        };

        let err = source
            .source_ref("/example/other/file.md", PathPrivacy::StoreRelativePath)
            .unwrap_err();
        assert_eq!(err.field, "path");
    }
}
