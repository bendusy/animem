use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaArtifactKind {
    RepeatableMigration,
    OneTimeMigration,
    ManualStep,
    Manifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaArtifactFile {
    pub file_id: String,
    pub kind: SchemaArtifactKind,
    pub sha256: String,
    pub order: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub source_repo: String,
    pub source_commit: String,
    pub files: Vec<SchemaArtifactFile>,
}

impl SchemaArtifact {
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_artifact_roundtrips_without_paths_or_sql_text() {
        let artifact = SchemaArtifact {
            schema_version: "animem.schema-artifact.v1".into(),
            artifact_id: "artifact_example_001".into(),
            source_repo: "example-schema-source".into(),
            source_commit: "0123456789abcdef".into(),
            files: vec![SchemaArtifactFile {
                file_id: "migration_001".into(),
                kind: SchemaArtifactKind::RepeatableMigration,
                sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                order: Some(1),
                reason: None,
            }],
        };

        let json = serde_json::to_string(&artifact).expect("serialize artifact");
        assert!(!json.contains("/"));
        assert!(!json.contains("SELECT"));

        let decoded: SchemaArtifact = serde_json::from_str(&json).expect("deserialize artifact");
        assert_eq!(decoded.file_count(), 1);
        assert_eq!(
            decoded.files[0].kind,
            SchemaArtifactKind::RepeatableMigration
        );
    }
}
