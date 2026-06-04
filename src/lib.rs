//! Public core primitives for document-grounded agent memory.
//!
//! The base crate is intentionally storage-free and network-free. Callers own
//! persistence, embedding, and LLM integration.

#![forbid(unsafe_code)]

mod candidate;
mod document;
mod error;
mod extension;
mod ids;
mod maintenance;
mod profile;
mod schema;
mod splitter;
mod store;

pub use candidate::{Candidate, CandidateKind, CandidateStatus, EvidenceSpan};
pub use document::{AssetKind, DocumentAsset, DocumentCard, DocumentSection};
pub use error::{AnimemError, Result};
pub use extension::{
    CandidateTypeMapping, CardRulePack, ExtensionProfile, PromotionPolicy, TextPattern,
    TokenizerConfig,
};
pub use ids::{asset_id, section_id};
pub use maintenance::{MaintenanceJob, MaintenancePlan};
pub use profile::{
    DataSource, LocalProfile, MaintenancePolicy, PathPrivacy, ProfileValidationError,
};
pub use schema::{SchemaArtifact, SchemaArtifactFile, SchemaArtifactKind};
pub use splitter::{split_sections, SplitOptions};
pub use store::{
    DocumentAssetFilter, DocumentAssetPage, DocumentSearchHit, DocumentSearchHitKind,
    DocumentSearchRequest, DocumentSearchResult, DocumentSearchStore, DocumentStore, StoreError,
    StoreFuture,
};
