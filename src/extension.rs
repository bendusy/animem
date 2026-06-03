use serde::{Deserialize, Serialize};

use crate::{CandidateKind, ProfileValidationError};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionProfile {
    pub name: String,
    #[serde(default)]
    pub default_library: Option<String>,
    #[serde(default)]
    pub tokenizer: TokenizerConfig,
    #[serde(default)]
    pub card_rules: CardRulePack,
    #[serde(default)]
    pub promotion: PromotionPolicy,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizerConfig {
    #[serde(default)]
    pub custom_terms: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRulePack {
    #[serde(default)]
    pub organization_terms: Vec<String>,
    #[serde(default)]
    pub document_type_patterns: Vec<TextPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPattern {
    pub contains: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionPolicy {
    #[serde(default)]
    pub candidate_type_mappings: Vec<CandidateTypeMapping>,
    #[serde(default)]
    pub source_agent: Option<String>,
    #[serde(default)]
    pub context_project: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateTypeMapping {
    pub candidate_kind: CandidateKind,
    pub experience_type: String,
}

impl ExtensionProfile {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_optional_text("name", Some(&self.name))?;
        validate_optional_text("default_library", self.default_library.as_deref())?;
        self.tokenizer.validate()?;
        self.card_rules.validate()?;
        self.promotion.validate()?;
        Ok(())
    }

    pub fn experience_type_for(&self, kind: CandidateKind) -> Option<&str> {
        self.promotion
            .candidate_type_mappings
            .iter()
            .find(|mapping| mapping.candidate_kind == kind)
            .map(|mapping| mapping.experience_type.as_str())
    }
}

impl TokenizerConfig {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_list("tokenizer.custom_terms", &self.custom_terms)
    }
}

impl CardRulePack {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_list("card_rules.organization_terms", &self.organization_terms)?;
        for pattern in &self.document_type_patterns {
            validate_optional_text(
                "card_rules.document_type_patterns.contains",
                Some(&pattern.contains),
            )?;
            validate_optional_text(
                "card_rules.document_type_patterns.value",
                Some(&pattern.value),
            )?;
        }
        Ok(())
    }
}

impl PromotionPolicy {
    pub fn validate(&self) -> std::result::Result<(), ProfileValidationError> {
        validate_optional_text("promotion.source_agent", self.source_agent.as_deref())?;
        validate_optional_text("promotion.context_project", self.context_project.as_deref())?;
        for mapping in &self.candidate_type_mappings {
            validate_optional_text(
                "promotion.candidate_type_mappings.experience_type",
                Some(&mapping.experience_type),
            )?;
        }
        Ok(())
    }
}

fn validate_list(
    field: &'static str,
    values: &[String],
) -> std::result::Result<(), ProfileValidationError> {
    for value in values {
        validate_optional_text(field, Some(value))?;
    }
    Ok(())
}

fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
) -> std::result::Result<(), ProfileValidationError> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(ProfileValidationError {
                field,
                message: "must not be empty when set".to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_synthetic_extension_profile() {
        let profile = ExtensionProfile {
            name: "example-extension".to_string(),
            default_library: Some("documents".to_string()),
            tokenizer: TokenizerConfig {
                custom_terms: vec!["Project Alpha".to_string()],
            },
            card_rules: CardRulePack {
                organization_terms: vec!["Example Org".to_string()],
                document_type_patterns: vec![TextPattern {
                    contains: "memo".to_string(),
                    value: "memo".to_string(),
                }],
            },
            promotion: PromotionPolicy {
                candidate_type_mappings: vec![CandidateTypeMapping {
                    candidate_kind: CandidateKind::Procedure,
                    experience_type: "procedure".to_string(),
                }],
                source_agent: Some("example-adapter".to_string()),
                context_project: Some("example-project".to_string()),
            },
        };

        profile.validate().unwrap();
        assert_eq!(
            profile.experience_type_for(CandidateKind::Procedure),
            Some("procedure")
        );
        assert_eq!(profile.experience_type_for(CandidateKind::Fact), None);
    }

    #[test]
    fn rejects_empty_custom_terms() {
        let profile = ExtensionProfile {
            name: "example-extension".to_string(),
            tokenizer: TokenizerConfig {
                custom_terms: vec![" ".to_string()],
            },
            ..Default::default()
        };

        let err = profile.validate().unwrap_err();
        assert_eq!(err.field, "tokenizer.custom_terms");
    }
}
