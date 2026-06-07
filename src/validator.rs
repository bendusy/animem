//! Public validation primitives — observation markers, secret scanning, hashing.
//!
//! Storage-free utilities for experience body validation.
//! The full `validate_write` function (with LibraryRegistry) stays in am-core.

use std::sync::OnceLock;

use sha2::{Digest, Sha256};

// ============================================================================
// R-1: observation markers
// ============================================================================

/// Body must contain at least one of these markers.
pub const OBSERVATION_MARKERS: &[&str] = &["[要点]", "[坑]", "[范式]", "[决策]"];

/// Check whether body contains any observation marker.
pub fn has_observation_marker(body: &str) -> bool {
    OBSERVATION_MARKERS.iter().any(|m| body.contains(m))
}

// ============================================================================
// R-5: secret patterns
// ============================================================================

#[derive(Debug)]
pub struct SecretPatterns {
    patterns: Vec<(String, regex::Regex)>,
}

fn secret_patterns() -> &'static SecretPatterns {
    static INST: OnceLock<SecretPatterns> = OnceLock::new();
    INST.get_or_init(|| {
        let raw: &[(&str, &str)] = &[
            ("Bearer Token", r"Bearer\s+[A-Za-z0-9._\-]{20,}"),
            ("OpenAI Key", r"sk-[A-Za-z0-9]{20,}"),
            ("AWS Access", r"AKIA[A-Z0-9]{16}"),
            ("PrivateKey", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
            ("GitHub Token", r"gh[posru]_[A-Za-z0-9]{36,}"),
            ("GitLab Token", r"glpat-[A-Za-z0-9_\-]{20,}"),
            ("Slack Token", r"xox[baprs]-[A-Za-z0-9\-]{10,}"),
            ("Google API Key", r"AIza[A-Za-z0-9_\-]{35}"),
            ("Anthropic Key", r"sk-ant-[A-Za-z0-9_\-]{20,}"),
            (
                "Generic Secret Assignment",
                r#"(?i)(api[_\-]?key|secret|token|passwd|password)\s*[=:]\s*["']?[A-Za-z0-9._\-/+]{16,}"#,
            ),
        ];
        let patterns = raw
            .iter()
            .map(|(name, pat)| {
                (
                    name.to_string(),
                    regex::Regex::new(pat).expect("regex compile"),
                )
            })
            .collect();
        SecretPatterns { patterns }
    })
}

/// Scan body for secret patterns. Returns (kind, detail) on first match.
pub fn match_any_secret(body: &str) -> Option<(String, String)> {
    for (kind, re) in &secret_patterns().patterns {
        if let Some(m) = re.find(body) {
            let snippet = m.as_str();
            let detail = if snippet.len() > 60 {
                format!("{}…", &snippet[..60])
            } else {
                snippet.to_string()
            };
            return Some((kind.clone(), detail));
        }
    }
    None
}

// ============================================================================
// Hashing
// ============================================================================

/// SHA-256 hex digest of body text.
pub fn sha256_hex(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_marker_detected() {
        assert!(has_observation_marker("[要点] summary"));
        assert!(has_observation_marker("text [决策] more"));
        assert!(!has_observation_marker("no marker here"));
    }

    #[test]
    fn secret_scan_finds_token() {
        // Use a pattern that detection rules catch but doesn't trigger the
        // source-bundle secret scanner on test literals.
        let body = "xoxb-1234567890abcdef";
        let result = match_any_secret(body);
        assert!(result.is_some());
    }

    #[test]
    fn secret_scan_clean_body() {
        assert!(match_any_secret("normal text").is_none());
    }

    #[test]
    fn sha256_consistent() {
        let h1 = sha256_hex("hello");
        let h2 = sha256_hex("hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}
