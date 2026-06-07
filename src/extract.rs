//! LLM extraction contract — request/response shapes.
//!
//! These types define the public API contract for document-grounded
//! candidate extraction. Private runtimes implement the actual LLM
//! call and conversion logic.

use serde::{Deserialize, Serialize};

use crate::candidate::Candidate;

/// Extraction request — text content with document grounding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    /// Section or document text to extract from.
    pub text: String,
    /// Source document asset identifier.
    pub asset_id: String,
    /// Optional card identifier for grouped extraction.
    pub card_id: Option<String>,
}

/// Extraction result — candidates produced by an LLM extraction pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResult {
    /// Candidates extracted from the source text.
    pub candidates: Vec<Candidate>,
    /// Model identifier used for extraction.
    pub model_used: String,
    /// Approximate token count consumed.
    pub tokens_used: usize,
    /// Extraction duration in milliseconds.
    pub duration_ms: u64,
}
