//! Object authority level — storage-free enum + parser.
//!
//! Public core type. Storage-free, network-free. The PG CHECK string values
//! (`as_str`) are a stable contract consumed by the private adapter.
//! Defensive downgrade of unknown values is the caller's responsibility:
//! `parse_opt_authority` returns `None` for unknown input WITHOUT logging,
//! so the public core stays free of any logging/runtime dependency. The
//! private adapter may log the downgrade.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Object authority level (four tiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Authority {
    /// Candidate: freshly observed / agent-guessed / background-extracted.
    /// Not injected, does not affect decisions.
    Candidate,
    /// Reused/confirmed >= 1 time. Enters the regular recall pool.
    Observed,
    /// Stable memory. Same recall pool as observed in stage 1.
    Established,
    /// Hard constraint / core value. Force-injected, may block behavior.
    Authoritative,
}

impl Authority {
    /// String value used in the PG CHECK constraint.
    pub fn as_str(&self) -> &'static str {
        match self {
            Authority::Candidate => "candidate",
            Authority::Observed => "observed",
            Authority::Established => "established",
            Authority::Authoritative => "authoritative",
        }
    }
}

impl std::fmt::Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Authority parse error.
#[derive(Debug, Error)]
#[error("unknown authority value: {0:?}")]
pub struct UnknownAuthority(pub String);

impl TryFrom<&str> for Authority {
    type Error = UnknownAuthority;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "candidate" => Ok(Authority::Candidate),
            "observed" => Ok(Authority::Observed),
            "established" => Ok(Authority::Established),
            "authoritative" => Ok(Authority::Authoritative),
            other => Err(UnknownAuthority(other.to_string())),
        }
    }
}

/// Defensive parse from `Option<String>` (e.g. md frontmatter field).
///
/// - `None` -> `None`.
/// - `Some(valid)` -> `Some(Authority::*)`.
/// - `Some(invalid)` -> `None` (defensive downgrade, NO logging here).
///
/// The orphan rule forbids `impl TryFrom<Option<String>> for Option<Authority>`,
/// so this is a free function. Callers that need to log the downgrade can use
/// `Authority::try_from` directly and handle the `Err`.
pub fn parse_opt_authority(opt: Option<String>) -> Option<Authority> {
    opt.and_then(|s| Authority::try_from(s.as_str()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_as_str_try_from() {
        for a in [
            Authority::Candidate,
            Authority::Observed,
            Authority::Established,
            Authority::Authoritative,
        ] {
            assert_eq!(Authority::try_from(a.as_str()).unwrap(), a);
        }
    }

    #[test]
    fn unknown_is_err_and_downgrades_to_none() {
        assert!(Authority::try_from("bogus").is_err());
        assert_eq!(parse_opt_authority(Some("bogus".into())), None);
        assert_eq!(parse_opt_authority(None), None);
        assert_eq!(
            parse_opt_authority(Some("observed".into())),
            Some(Authority::Observed)
        );
    }

    #[test]
    fn serde_lowercase() {
        let json = serde_json::to_string(&Authority::Authoritative).unwrap();
        assert_eq!(json, "\"authoritative\"");
    }
}
