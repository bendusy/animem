//! Library / ExperienceType runtime registry — storage-free classification.
//!
//! LibraryRegistry maps library names to their configuration
//! (type allowlists, lint rules, feature flags).
//! I/O methods (from_yaml_file, from_env) stay in am-core.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{DomainError, DomainResult, ExperienceType, Library};

// ============================================================================
// LibraryConfig — per-library metadata
// ============================================================================

/// Configuration for a single library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryConfig {
    /// Library name (Chinese, matches PG experience.library).
    pub name: String,
    /// Enable secret-word scanning (R-5 sub-rule).
    #[serde(default)]
    pub lint: bool,
    /// Enable banwen-flow integration.
    #[serde(default)]
    pub banwen: bool,
    /// Allowed experience types for this library. None = unrestricted.
    #[serde(default)]
    pub type_allowlist: Option<Vec<String>>,
}

// ============================================================================
// ConfigFile — YAML top-level
// ============================================================================

/// Top-level config file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub libraries: Vec<LibraryConfig>,
}

// ============================================================================
// LibraryRegistry
// ============================================================================

/// Runtime registry of library configurations.
#[derive(Debug, Clone)]
pub struct LibraryRegistry {
    libraries: HashMap<String, LibraryConfig>,
}

impl LibraryRegistry {
    /// Construct from ConfigFile.
    pub fn from_config(cfg: ConfigFile) -> DomainResult<Self> {
        let mut libraries = HashMap::with_capacity(cfg.libraries.len());
        for lib in cfg.libraries {
            if libraries.contains_key(&lib.name) {
                return Err(DomainError::Config(format!(
                    "library '{}' duplicated in config",
                    lib.name
                )));
            }
            libraries.insert(lib.name.clone(), lib);
        }
        if libraries.is_empty() {
            return Err(DomainError::Config(
                "library registry empty: at least one library required".into(),
            ));
        }
        Ok(LibraryRegistry { libraries })
    }

    /// Built-in defaults (6 historical libraries from mf).
    pub fn builtin_defaults() -> Self {
        let libraries = vec![
            LibraryConfig {
                name: "办文".to_string(),
                lint: true,
                banwen: true,
                type_allowlist: Some(vec![
                    "范文".into(),
                    "改稿".into(),
                    "流程".into(),
                    "任务".into(),
                    "决策".into(),
                    "范式".into(),
                    "规则".into(),
                    "经验".into(),
                ]),
            },
            LibraryConfig {
                name: "技术".into(),
                lint: false,
                banwen: false,
                type_allowlist: None,
            },
            LibraryConfig {
                name: "决策".into(),
                lint: false,
                banwen: false,
                type_allowlist: None,
            },
            LibraryConfig {
                name: "生活".into(),
                lint: false,
                banwen: false,
                type_allowlist: None,
            },
            LibraryConfig {
                name: "学习".into(),
                lint: false,
                banwen: false,
                type_allowlist: None,
            },
            LibraryConfig {
                name: "副业".into(),
                lint: false,
                banwen: false,
                type_allowlist: None,
            },
        ];
        Self::from_config(ConfigFile { libraries }).expect("builtin defaults always valid")
    }

    /// Check whether a library is registered.
    pub fn contains(&self, lib: &Library) -> bool {
        self.libraries.contains_key(lib.as_str())
    }

    /// Get the full config for a library.
    pub fn config_for(&self, lib: &Library) -> Option<&LibraryConfig> {
        self.libraries.get(lib.as_str())
    }

    /// All library names (for UI / debug).
    pub fn names(&self) -> Vec<String> {
        self.libraries.keys().cloned().collect()
    }

    /// Validate that a library is registered.
    pub fn validate(&self, lib: &Library) -> DomainResult<()> {
        if self.contains(lib) {
            Ok(())
        } else {
            Err(DomainError::InvalidLibrary(lib.as_str().to_string()))
        }
    }

    /// Check if an experience type is allowed in this library.
    pub fn type_allowed(&self, lib: &Library, type_: &ExperienceType) -> bool {
        match self.config_for(lib) {
            Some(cfg) => match &cfg.type_allowlist {
                Some(allowed) => allowed.iter().any(|a| a == type_.as_str()),
                None => true,
            },
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_has_6_libs() {
        let r = LibraryRegistry::builtin_defaults();
        assert_eq!(r.names().len(), 6);
        assert!(r.contains(&Library::from_raw("技术")));
        assert!(r.contains(&Library::from_raw("决策")));
        assert!(!r.contains(&Library::from_raw("家庭")));
    }

    #[test]
    fn banwen_has_allowlist() {
        let r = LibraryRegistry::builtin_defaults();
        let cfg = r.config_for(&Library::from_raw("办文")).unwrap();
        assert!(cfg.lint);
        assert!(cfg.banwen);
        assert_eq!(cfg.type_allowlist.as_ref().unwrap().len(), 8);
    }

    #[test]
    fn tech_no_allowlist() {
        let r = LibraryRegistry::builtin_defaults();
        let cfg = r.config_for(&Library::from_raw("技术")).unwrap();
        assert!(!cfg.lint);
        assert!(!cfg.banwen);
        assert!(cfg.type_allowlist.is_none());
    }

    #[test]
    fn from_yaml_roundtrip() {
        let yaml = r#"
libraries:
  - name: 家庭
    lint: false
    banwen: false
  - name: 阅读
    type_allowlist: [书评, 摘录]
"#;
        let cfg: ConfigFile = serde_yaml::from_str(yaml).unwrap();
        let r = LibraryRegistry::from_config(cfg).unwrap();
        assert_eq!(r.names().len(), 2);
        assert!(r.contains(&Library::from_raw("家庭")));
        assert!(r.contains(&Library::from_raw("阅读")));
        let reading = r.config_for(&Library::from_raw("阅读")).unwrap();
        assert_eq!(reading.type_allowlist.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn validate_rejects_unknown_library() {
        let r = LibraryRegistry::builtin_defaults();
        let bad = Library::from_raw("不存在的库");
        let res = r.validate(&bad);
        assert!(matches!(res, Err(DomainError::InvalidLibrary(_))));
    }

    #[test]
    fn duplicate_library_rejected() {
        let cfg = ConfigFile {
            libraries: vec![
                LibraryConfig {
                    name: "a".into(),
                    lint: false,
                    banwen: false,
                    type_allowlist: None,
                },
                LibraryConfig {
                    name: "a".into(),
                    lint: false,
                    banwen: false,
                    type_allowlist: None,
                },
            ],
        };
        assert!(LibraryRegistry::from_config(cfg).is_err());
    }

    #[test]
    fn empty_config_rejected() {
        let cfg = ConfigFile { libraries: vec![] };
        assert!(LibraryRegistry::from_config(cfg).is_err());
    }

    #[test]
    fn type_allowed_unrestricted() {
        let r = LibraryRegistry::builtin_defaults();
        assert!(r.type_allowed(
            &Library::from_raw("技术"),
            &ExperienceType::from_raw("anything")
        ));
    }

    #[test]
    fn type_allowed_restricted() {
        let r = LibraryRegistry::builtin_defaults();
        assert!(r.type_allowed(
            &Library::from_raw("办文"),
            &ExperienceType::from_raw("范文")
        ));
        assert!(!r.type_allowed(
            &Library::from_raw("办文"),
            &ExperienceType::from_raw("unknown")
        ));
    }
}
