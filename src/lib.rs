//! Public core primitives for document-grounded agent memory.
//!
//! The base crate is intentionally storage-free and network-free. Callers own
//! persistence, embedding, and LLM integration.

#![forbid(unsafe_code)]

mod candidate;
mod document;
mod error;
mod ids;
mod maintenance;
mod profile;
mod splitter;

pub use candidate::{Candidate, CandidateKind, CandidateStatus, EvidenceSpan};
pub use document::{AssetKind, DocumentAsset, DocumentCard, DocumentSection};
pub use error::{AnimemError, Result};
pub use ids::{asset_id, section_id};
pub use maintenance::{MaintenanceJob, MaintenancePlan};
pub use profile::{
    DataSource, LocalProfile, MaintenancePolicy, PathPrivacy, ProfileValidationError,
};
pub use splitter::{split_sections, SplitOptions};
