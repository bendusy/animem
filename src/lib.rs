//! Public core primitives for document-grounded agent memory.
//!
//! The base crate is intentionally storage-free and network-free. Callers own
//! persistence, embedding, and LLM integration.

#![forbid(unsafe_code)]

mod authority;
mod candidate;
mod document;
mod domain;
mod error;
mod extension;
mod extract;
mod ids;
mod maintenance;
mod profile;
mod provenance;
mod registry;
mod schema;
mod splitter;
mod store;
mod validator;

pub use authority::{parse_opt_authority, Authority, UnknownAuthority};
pub use candidate::{Candidate, CandidateKind, CandidateStatus, EvidenceSpan};
pub use document::{AssetKind, DocumentAsset, DocumentCard, DocumentSection};
pub use domain::{
    AccessCount, Confidence, ContentHash, DomainError, DomainResult, ExperienceType, HitsCount,
    Library, NonEmptyString, Slug, SourcesCount, Tag,
};
pub use error::{AnimemError, Result};
pub use extension::{
    CandidateTypeMapping, CardRulePack, ExtensionProfile, PromotionPolicy, TextPattern,
    TokenizerConfig,
};
pub use extract::{ExtractRequest, ExtractResult};
pub use ids::{asset_id, section_id};
pub use maintenance::{MaintenanceJob, MaintenancePlan};
pub use profile::{
    DataSource, LocalProfile, MaintenancePolicy, PathPrivacy, ProfileValidationError,
};
pub use provenance::{
    ProvenanceEvent, ProvenanceEventKind, ProvenanceRef, ProvenanceRefKind, RedactedField,
    RedactionState, RedactionSummary, PROVENANCE_EVENT_SCHEMA_VERSION,
};
pub use registry::{ConfigFile, LibraryConfig, LibraryRegistry};
pub use schema::{SchemaArtifact, SchemaArtifactFile, SchemaArtifactKind};
pub use splitter::{split_sections, SplitOptions};
pub use store::{
    DocumentAssetFilter, DocumentAssetPage, DocumentSearchHit, DocumentSearchHitKind,
    DocumentSearchRequest, DocumentSearchResult, DocumentSearchStore, DocumentStore, StoreError,
    StoreFuture,
};
pub use validator::{has_observation_marker, match_any_secret, sha256_hex, OBSERVATION_MARKERS};
