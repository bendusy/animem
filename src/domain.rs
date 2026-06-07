//! Public domain types — storage-free value objects for agent memory.
//!
//! These types define the core vocabulary of the memory system:
//! identifiers (Slug), classification (Library, ExperienceType),
//! and content addressing (ContentHash).
//!
//! All types are storage-free and network-free by default.
//! Enable feature `db` for sqlx::Type derives (private downstream only).

use serde::{Deserialize, Serialize};

// ============================================================================
// Identifier types
// ============================================================================

/// Unique experience identifier (kebab-case, ≤ 256 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct Slug(String);

impl Slug {
    pub fn new(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into();
        if s.is_empty() {
            return Err(DomainError::InvalidSlug("empty".into()));
        }
        if s.len() > 256 {
            return Err(DomainError::InvalidSlug(format!(
                "too long: {} chars",
                s.len()
            )));
        }
        Ok(Slug(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
}

// ============================================================================
// Classification types
// ============================================================================

/// Memory library (closed set: 办文/技术/决策/生活/学习/副业).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct Library(String);

/// Experience type within a library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct ExperienceType(String);

// ============================================================================
// Content types
// ============================================================================

/// SHA-256 content hash for deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct ContentHash(String);

impl ContentHash {
    pub fn new(hex: String) -> Result<Self, DomainError> {
        if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::InvalidContentHash(hex));
        }
        Ok(ContentHash(hex))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Non-empty string (validated at construction).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(label: &str, s: &str) -> Result<Self, DomainError> {
        if s.trim().is_empty() {
            return Err(DomainError::InvalidNonEmpty(label.into(), s.into()));
        }
        Ok(NonEmptyString(s.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tag (free-form label).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct Tag(pub String);

// ============================================================================
// Metrics types
// ============================================================================

/// Confidence score (0.0..=1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct Confidence(f32);

/// Sources count (≥ 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct SourcesCount(i32);

/// Access count (≥ 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct AccessCount(i32);

/// Recall hit count (≥ 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct HitsCount(i32);

// ============================================================================
// Error type
// ============================================================================

/// Domain validation errors.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid slug: '{0}'")]
    InvalidSlug(String),

    #[error("invalid library: '{0}'")]
    InvalidLibrary(String),

    #[error("invalid experience type: '{0}'")]
    InvalidExperienceType(String),

    #[error("invalid content hash: '{0}'")]
    InvalidContentHash(String),

    #[error("invalid non-empty string '{0}': '{1}'")]
    InvalidNonEmpty(String, String),
}

pub type DomainResult<T> = Result<T, DomainError>;
