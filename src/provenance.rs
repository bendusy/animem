use serde::{Deserialize, Serialize};

use crate::EvidenceSpan;

pub const PROVENANCE_EVENT_SCHEMA_VERSION: &str = "animem.provenance.event.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceEventKind {
    CandidateEvidence,
    ReviewDecision,
    MemoryWritten,
    SchemaArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceRefKind {
    DocumentAsset,
    DocumentSection,
    Candidate,
    MemoryRecord,
    SchemaArtifact,
    Project,
    Runtime,
    Actor,
    Artifact,
    Reason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub ref_id: String,
    pub kind: ProvenanceRefKind,
    pub id: String,
    pub content_hash: Option<String>,
    pub span: Option<EvidenceSpan>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceEvent {
    pub schema_version: String,
    pub event_id: String,
    pub event_kind: ProvenanceEventKind,
    #[serde(default)]
    pub occurred_at: String,
    pub subject: ProvenanceRef,
    #[serde(default)]
    pub project_ref: Option<ProvenanceRef>,
    #[serde(default)]
    pub runtime_ref: Option<ProvenanceRef>,
    #[serde(default)]
    pub actor_ref: Option<ProvenanceRef>,
    #[serde(default)]
    pub artifact_ref: Option<ProvenanceRef>,
    #[serde(default)]
    pub reason_ref: Option<ProvenanceRef>,
    pub inputs: Vec<ProvenanceRef>,
    pub outputs: Vec<ProvenanceRef>,
    #[serde(default)]
    pub redaction_status: RedactionState,
    pub redaction: RedactionSummary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionState {
    #[default]
    Synthetic,
    Redacted,
    HashOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedField {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionSummary {
    pub state: RedactionState,
    pub policy: String,
    pub removed: Vec<RedactedField>,
}

impl ProvenanceEvent {
    pub fn synthetic(
        event_id: impl Into<String>,
        event_kind: ProvenanceEventKind,
        subject: ProvenanceRef,
    ) -> Self {
        Self {
            schema_version: PROVENANCE_EVENT_SCHEMA_VERSION.into(),
            event_id: event_id.into(),
            event_kind,
            occurred_at: "1970-01-01T00:00:00Z".into(),
            subject,
            project_ref: None,
            runtime_ref: None,
            actor_ref: None,
            artifact_ref: None,
            reason_ref: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            redaction_status: RedactionState::Synthetic,
            redaction: RedactionSummary::synthetic(),
        }
    }
}

impl RedactionSummary {
    pub fn synthetic() -> Self {
        Self {
            state: RedactionState::Synthetic,
            policy: "public-example-v1".into(),
            removed: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref() -> ProvenanceRef {
        ProvenanceRef {
            ref_id: "ref_example_section_001".into(),
            kind: ProvenanceRefKind::DocumentSection,
            id: "section:example-policy-memo-a:001".into(),
            content_hash: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            span: Some(EvidenceSpan {
                section_id: "section:example-policy-memo-a:001".into(),
                char_start: 120,
                char_end: 184,
            }),
            label: Some("Policy Memo A section 1".into()),
        }
    }

    #[test]
    fn provenance_event_roundtrips_without_sensitive_field_names() {
        let event = ProvenanceEvent::synthetic(
            "event_example_001",
            ProvenanceEventKind::CandidateEvidence,
            sample_ref(),
        );
        let json = serde_json::to_string(&event).expect("serialize provenance event");

        for forbidden in [
            "prompt",
            "system_prompt",
            "transcript",
            "cwd",
            "source_path",
            "document_text",
            "provider_endpoint",
        ] {
            assert!(!json.contains(forbidden), "unexpected field: {forbidden}");
        }
        assert!(json.contains(PROVENANCE_EVENT_SCHEMA_VERSION));
        assert!(json.contains("occurred_at"));
        assert!(json.contains("project_ref"));
        assert!(json.contains("runtime_ref"));
        assert!(json.contains("actor_ref"));
        assert!(json.contains("artifact_ref"));
        assert!(json.contains("reason_ref"));
        assert!(json.contains("redaction_status"));

        let decoded: ProvenanceEvent =
            serde_json::from_str(&json).expect("decode provenance event");
        assert_eq!(decoded.event_kind, ProvenanceEventKind::CandidateEvidence);
        assert_eq!(decoded.subject.kind, ProvenanceRefKind::DocumentSection);
        assert_eq!(decoded.redaction_status, RedactionState::Synthetic);
    }
}
