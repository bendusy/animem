use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Text,
    Markdown,
    Pdf,
    Docx,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentAsset {
    pub asset_id: String,
    pub kind: AssetKind,
    pub source_label: Option<String>,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSection {
    pub section_id: String,
    pub asset_id: String,
    pub ordinal: usize,
    pub heading: Option<String>,
    pub text: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCard {
    pub asset_id: String,
    pub title: Option<String>,
    pub kind: AssetKind,
    pub topic_tags: Vec<String>,
    pub summary: Option<String>,
}

impl DocumentCard {
    pub fn minimal(asset: &DocumentAsset) -> Self {
        Self {
            asset_id: asset.asset_id.clone(),
            title: asset.source_label.clone(),
            kind: asset.kind,
            topic_tags: Vec::new(),
            summary: None,
        }
    }
}
