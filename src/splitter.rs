use regex::Regex;
use std::sync::LazyLock;

use crate::document::DocumentSection;
use crate::error::{AnimemError, Result};
use crate::ids;

static MARKDOWN_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#{1,6}\s+(.+)$").expect("valid regex"));
static NUMBERED_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+\.|[A-Z]\.|[IVX]+\.)\s+(.+)$").expect("valid regex"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitOptions {
    pub min_section_chars: usize,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self {
            min_section_chars: 40,
        }
    }
}

pub fn split_sections(
    asset_id: &str,
    text: &str,
    options: SplitOptions,
) -> Result<Vec<DocumentSection>> {
    if text.trim().is_empty() {
        return Err(AnimemError::EmptyInput);
    }

    let headings = heading_offsets(text);
    if headings.is_empty() {
        return Ok(vec![single_section(asset_id, text)]);
    }

    let char_positions = byte_to_char_positions(text);
    let mut sections = Vec::new();
    for (idx, (byte_start, heading)) in headings.iter().enumerate() {
        let byte_end = headings
            .get(idx + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(text.len());
        let body = text[*byte_start..byte_end].trim();
        if body.chars().count() < options.min_section_chars && sections.is_empty() {
            continue;
        }
        let char_start = char_positions[*byte_start];
        let char_end = char_positions[byte_end];
        sections.push(DocumentSection {
            section_id: ids::section_id(asset_id, sections.len() + 1),
            asset_id: asset_id.to_string(),
            ordinal: sections.len() + 1,
            heading: Some(heading.clone()),
            text: body.to_string(),
            char_start,
            char_end,
        });
    }

    if sections.is_empty() {
        return Ok(vec![single_section(asset_id, text)]);
    }
    Ok(sections)
}

fn single_section(asset_id: &str, text: &str) -> DocumentSection {
    DocumentSection {
        section_id: ids::section_id(asset_id, 1),
        asset_id: asset_id.to_string(),
        ordinal: 1,
        heading: None,
        text: text.trim().to_string(),
        char_start: 0,
        char_end: text.chars().count(),
    }
}

fn heading_offsets(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in text.lines() {
        let heading = MARKDOWN_HEADING
            .captures(line)
            .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .or_else(|| {
                NUMBERED_HEADING
                    .captures(line)
                    .and_then(|c| c.get(2).map(|m| m.as_str().trim().to_string()))
            });
        if let Some(heading) = heading {
            out.push((offset, heading));
        }
        offset += line.len() + 1;
    }
    out
}

fn byte_to_char_positions(text: &str) -> Vec<usize> {
    let mut positions = vec![0; text.len() + 1];
    let mut char_idx = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        positions[byte_idx] = char_idx;
        for pos in positions
            .iter_mut()
            .take(byte_idx + ch.len_utf8())
            .skip(byte_idx + 1)
        {
            *pos = char_idx + 1;
        }
        char_idx += 1;
    }
    positions[text.len()] = char_idx;
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_markdown_sections() {
        let text = "# Overview\nProject Alpha overview has enough text to remain a section.\n# Decision\nProject Beta decision has enough text to remain a section.";
        let sections = split_sections("asset-1", text, SplitOptions::default()).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading.as_deref(), Some("Overview"));
        assert_eq!(sections[1].heading.as_deref(), Some("Decision"));
    }

    #[test]
    fn preserves_utf8_character_offsets() {
        let text = "# Summary\nExample Org uses cafe notes and generic policy text.\n# Details\nProject Alpha includes synthetic UTF-8 text: 概要資料.";
        let sections = split_sections("asset-1", text, SplitOptions::default()).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(
            sections[1].char_start,
            text[..text.find("# Details").unwrap()].chars().count()
        );
        assert_eq!(sections[1].char_end, text.chars().count());
    }

    #[test]
    fn returns_single_section_without_headings() {
        let text = "A synthetic paragraph for Example Org without any heading.";
        let sections = split_sections("asset-1", text, SplitOptions::default()).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, None);
    }

    #[test]
    fn rejects_empty_input() {
        let err = split_sections("asset-1", "   ", SplitOptions::default()).unwrap_err();
        assert_eq!(err, AnimemError::EmptyInput);
    }
}
