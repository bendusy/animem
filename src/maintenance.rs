use serde::{Deserialize, Serialize};

use crate::profile::{LocalProfile, PathPrivacy, ProfileValidationError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenancePlan {
    pub profile_name: String,
    pub dry_run_required: bool,
    pub jobs: Vec<MaintenanceJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceJob {
    pub source_id: String,
    pub label: String,
    pub root: String,
    pub batch: String,
    pub include_extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub path_privacy: PathPrivacy,
    pub write_source_paths: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPlan {
    pub profile_name: String,
    pub dry_run_required: bool,
    pub jobs: Vec<ScanJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanJob {
    pub source_id: String,
    pub label: String,
    pub batch: String,
    pub include_extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub path_privacy: PathPrivacy,
    pub write_source_paths: bool,
}

impl MaintenancePlan {
    pub fn from_profile(profile: &LocalProfile) -> Result<Self, ProfileValidationError> {
        profile.validate()?;
        let jobs = profile
            .sources
            .iter()
            .map(|source| MaintenanceJob {
                source_id: source.source_id.clone(),
                label: source.label.clone(),
                root: source.root.clone(),
                batch: format!("{}-{}", profile.maintenance.batch_prefix, source.source_id),
                include_extensions: source.include_extensions.clone(),
                exclude_globs: source.exclude_globs.clone(),
                path_privacy: profile.path_privacy,
                write_source_paths: profile.maintenance.write_source_paths,
            })
            .collect();

        Ok(Self {
            profile_name: profile.name.clone(),
            dry_run_required: profile.maintenance.require_dry_run,
            jobs,
        })
    }
}

impl ScanPlan {
    pub fn from_profile(profile: &LocalProfile) -> Result<Self, ProfileValidationError> {
        profile.validate()?;
        let jobs = profile
            .sources
            .iter()
            .map(|source| ScanJob {
                source_id: source.source_id.clone(),
                label: source.label.clone(),
                batch: format!("{}-{}", profile.maintenance.batch_prefix, source.source_id),
                include_extensions: source.include_extensions.clone(),
                exclude_globs: source.exclude_globs.clone(),
                path_privacy: profile.path_privacy,
                write_source_paths: profile.maintenance.write_source_paths,
            })
            .collect();

        Ok(Self {
            profile_name: profile.name.clone(),
            dry_run_required: profile.maintenance.require_dry_run,
            jobs,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{DataSource, MaintenancePolicy};

    use super::*;

    #[test]
    fn builds_jobs_from_profile_sources() {
        let profile = LocalProfile {
            schema_version: Some("1".to_string()),
            name: "example".to_string(),
            sources: vec![DataSource {
                source_id: "notes".to_string(),
                label: "Notes".to_string(),
                root: "/example/private/notes".to_string(),
                include_extensions: vec!["md".to_string()],
                exclude_globs: vec!["**/draft/**".to_string()],
            }],
            maintenance: MaintenancePolicy {
                batch_prefix: "manual".to_string(),
                require_dry_run: true,
                write_source_paths: false,
            },
            path_privacy: PathPrivacy::StoreRelativePath,
        };

        let plan = MaintenancePlan::from_profile(&profile).unwrap();
        assert_eq!(plan.profile_name, "example");
        assert!(plan.dry_run_required);
        assert_eq!(plan.jobs[0].batch, "manual-notes");
        assert!(!plan.jobs[0].write_source_paths);
    }

    #[test]
    fn scan_plan_omits_source_roots() {
        let profile = LocalProfile {
            schema_version: Some("1".to_string()),
            name: "example".to_string(),
            sources: vec![DataSource {
                source_id: "notes".to_string(),
                label: "Notes".to_string(),
                root: "/example/private/notes".to_string(),
                include_extensions: vec!["md".to_string()],
                exclude_globs: vec!["**/draft/**".to_string()],
            }],
            maintenance: MaintenancePolicy {
                batch_prefix: "manual".to_string(),
                require_dry_run: true,
                write_source_paths: false,
            },
            path_privacy: PathPrivacy::StoreRelativePath,
        };

        let plan = ScanPlan::from_profile(&profile).unwrap();
        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(json["jobs"][0]["source_id"], "notes");
        assert!(json["jobs"][0].get("root").is_none());
    }
}
