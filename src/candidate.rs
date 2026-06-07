use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    Fact,
    Procedure,
    Template,
    Rule,
    Correction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    Candidate,
    Approved,
    Rejected,
    Promoted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSpan {
    pub section_id: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: String,
    pub kind: CandidateKind,
    pub title: String,
    pub payload: Value,
    pub evidence: Vec<EvidenceSpan>,
    pub status: CandidateStatus,
    /// Neutral document reference — set when candidate is grounded in a document asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Optional card reference for grouped extraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
}

impl Candidate {
    pub fn new(
        candidate_id: impl Into<String>,
        kind: CandidateKind,
        title: impl Into<String>,
        payload: Value,
        evidence: Vec<EvidenceSpan>,
    ) -> Self {
        Self {
            candidate_id: candidate_id.into(),
            kind,
            title: title.into(),
            payload,
            evidence,
            status: CandidateStatus::Candidate,
            asset_id: None,
            card_id: None,
        }
    }
}
