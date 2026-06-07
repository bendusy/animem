//! Memory classification vocabulary — source kind, directive strength, severity.
//!
//! Storage-free enum types for categorizing experience entries.
//! Part of the ANIMEM axis system (write-path metadata).

use serde::{Deserialize, Serialize};

// ============================================================================
// SourceKind — how was this experience produced?
// ============================================================================

/// Origin category of an experience entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Direct observation or note.
    Observation,
    /// Mistake or pitfall encountered.
    Pitfall,
    /// Explicit directive or constraint.
    Directive,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Observation => "observation",
            SourceKind::Pitfall => "pitfall",
            SourceKind::Directive => "directive",
        }
    }

    /// Parse from string. Unknown values return None.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "observation" => Some(SourceKind::Observation),
            "pitfall" => Some(SourceKind::Pitfall),
            "directive" => Some(SourceKind::Directive),
            _ => None,
        }
    }
}

// ============================================================================
// DirectiveStrength — how strong is this constraint?
// ============================================================================

/// Enforcement level of a directive or constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectiveStrength {
    /// Soft preference, advisory.
    Preference,
    /// Hard rule, must be enforced.
    Hard,
}

impl DirectiveStrength {
    pub fn as_str(self) -> &'static str {
        match self {
            DirectiveStrength::Preference => "preference",
            DirectiveStrength::Hard => "hard",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "preference" => Some(DirectiveStrength::Preference),
            "hard" => Some(DirectiveStrength::Hard),
            _ => None,
        }
    }
}

// ============================================================================
// Severity — how bad is this error/event?
// ============================================================================

/// Severity level for errors, incidents, or events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// No severity / informational.
    None,
    /// Minor issue, low impact.
    Minor,
    /// Major issue, significant impact.
    Major,
    /// Critical issue, service-level impact.
    Critical,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::None => "none",
            Severity::Minor => "minor",
            Severity::Major => "major",
            Severity::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Severity::None),
            "minor" => Some(Severity::Minor),
            "major" => Some(Severity::Major),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_roundtrip() {
        assert_eq!(SourceKind::Observation.as_str(), "observation");
        assert_eq!(SourceKind::parse("pitfall"), Some(SourceKind::Pitfall));
        assert_eq!(SourceKind::parse("unknown"), None);
    }

    #[test]
    fn directive_strength_roundtrip() {
        assert_eq!(DirectiveStrength::Hard.as_str(), "hard");
        assert_eq!(
            DirectiveStrength::parse("preference"),
            Some(DirectiveStrength::Preference)
        );
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Critical > Severity::Major);
        assert!(Severity::Minor > Severity::None);
    }

    #[test]
    fn severity_serde() {
        let s: Severity = serde_json::from_str("\"minor\"").unwrap();
        assert_eq!(s, Severity::Minor);
        let json = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
    }
}
