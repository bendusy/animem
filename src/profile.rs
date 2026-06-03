use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalProfile {
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
            name: "example".to_string(),
            sources: vec![],
            maintenance: MaintenancePolicy::default(),
            path_privacy: PathPrivacy::default(),
        };

        let err = profile.validate().unwrap_err();
        assert_eq!(err.field, "sources");
    }
}
