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
    pub subject: ProvenanceRef,
    pub inputs: Vec<ProvenanceRef>,
    pub outputs: Vec<ProvenanceRef>,
    pub redaction: RedactionSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionState {
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
            subject,
            inputs: Vec::new(),
            outputs: Vec::new(),
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

        let decoded: ProvenanceEvent =
            serde_json::from_str(&json).expect("decode provenance event");
        assert_eq!(decoded.event_kind, ProvenanceEventKind::CandidateEvidence);
        assert_eq!(decoded.subject.kind, ProvenanceRefKind::DocumentSection);
    }
}
