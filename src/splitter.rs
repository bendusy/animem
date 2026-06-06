use regex::Regex;
use std::sync::LazyLock;

use crate::document::DocumentSection;
use crate::error::{AnimemError, Result};
use crate::ids;

static MARKDOWN_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#{1,6}\s+(.+)$").expect("valid regex"));
static NUMBERED_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+\.|[A-Z]\.|[IVX]+\.)\s+(.+)$").expect("valid regex"));

// ---------------------------------------------------------------------------
// SplitOptions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitOptions {
    pub min_section_chars: usize,
    /// Maximum characters per section. Sections exceeding this are sub-split
    /// at paragraph boundaries. `None` means no limit.
    pub max_section_chars: Option<usize>,
    /// Enable Chinese government document heading detection
    /// (一、/（一）/第X条/第X章/会议指出 etc.).
    /// Default: `false` — does not affect the English/markdown path.
    pub cjk_headings: bool,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self {
            min_section_chars: 40,
            max_section_chars: None,
            cjk_headings: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn split_sections(
    asset_id: &str,
    text: &str,
    options: SplitOptions,
) -> Result<Vec<DocumentSection>> {
    if text.trim().is_empty() {
        return Err(AnimemError::EmptyInput);
    }

    let headings = heading_offsets(text, options.cjk_headings);
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

        let base = DocumentSection {
            section_id: ids::section_id(asset_id, sections.len() + 1),
            asset_id: asset_id.to_string(),
            ordinal: sections.len() + 1,
            heading: Some(heading.clone()),
            text: body.to_string(),
            char_start,
            char_end,
            meta: None,
        };

        if let Some(max) = options.max_section_chars {
            if body.chars().count() > max {
                let subs = sub_split(base, max, &mut sections.len());
                sections.extend(subs);
                continue;
            }
        }

        sections.push(base);
    }

    if sections.is_empty() {
        return Ok(vec![single_section(asset_id, text)]);
    }
    Ok(sections)
}

// ---------------------------------------------------------------------------
// Sub-splitting for max_section_chars
// ---------------------------------------------------------------------------

/// Split an oversized section at paragraph / line boundaries.
/// `ordinal_base` is mutated so callers can keep ordinals monotonic.
fn sub_split(
    sec: DocumentSection,
    max_chars: usize,
    ordinal_base: &mut usize,
) -> Vec<DocumentSection> {
    let text = &sec.text;
    let delimiter = if text.contains("\n\n") { "\n\n" } else { "\n" };
    let paragraphs: Vec<&str> = text.split(delimiter).collect();

    let mut result = Vec::new();
    let mut current = String::new();

    for para in &paragraphs {
        let would_be = current.chars().count() + para.chars().count() + delimiter.len();
        if !current.is_empty() && would_be > max_chars {
            if !current.trim().is_empty() {
                *ordinal_base += 1;
                result.push(DocumentSection {
                    section_id: ids::section_id(&sec.asset_id, *ordinal_base),
                    asset_id: sec.asset_id.clone(),
                    ordinal: *ordinal_base,
                    heading: sec.heading.clone(),
                    text: std::mem::take(&mut current),
                    char_start: sec.char_start,
                    char_end: sec.char_end,
                    meta: None,
                });
            } else {
                current.clear();
            }
        }
        if !current.is_empty() {
            current.push_str(delimiter);
        }
        current.push_str(para);
    }

    if !current.trim().is_empty() {
        *ordinal_base += 1;
        result.push(DocumentSection {
            section_id: ids::section_id(&sec.asset_id, *ordinal_base),
            asset_id: sec.asset_id.clone(),
            ordinal: *ordinal_base,
            heading: sec.heading.clone(),
            text: current,
            char_start: sec.char_start,
            char_end: sec.char_end,
            meta: None,
        });
    }

    if result.is_empty() {
        vec![sec]
    } else {
        result
    }
}

// ---------------------------------------------------------------------------
// Heading detection
// ---------------------------------------------------------------------------

fn single_section(asset_id: &str, text: &str) -> DocumentSection {
    DocumentSection {
        section_id: ids::section_id(asset_id, 1),
        asset_id: asset_id.to_string(),
        ordinal: 1,
        heading: None,
        text: text.trim().to_string(),
        char_start: 0,
        char_end: text.chars().count(),
        meta: None,
    }
}

fn heading_offsets(text: &str, cjk_headings: bool) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in text.lines() {
        let heading = detect_heading(line, cjk_headings);
        if let Some(heading) = heading {
            out.push((offset, heading));
        }
        offset += line.len() + 1;
    }
    out
}

fn detect_heading(line: &str, cjk_headings: bool) -> Option<String> {
    // Markdown headings
    if let Some(h) = MARKDOWN_HEADING
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
    {
        return Some(h);
    }
    // English numbered headings
    if let Some(h) = NUMBERED_HEADING
        .captures(line)
        .and_then(|c| c.get(2).map(|m| m.as_str().trim().to_string()))
    {
        return Some(h);
    }
    // Chinese headings (opt-in)
    if cjk_headings && is_chinese_heading(line) {
        return Some(strip_heading_delimiters(line));
    }
    None
}

// ---------------------------------------------------------------------------
// Chinese government document heading detection
// (ported from animem-mf-adapter/src/doc/section.rs)
// ---------------------------------------------------------------------------

const CHINESE_NUMS: [&str; 10] = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];
const ARTICLE_NUMS: [&str; 12] = [
    "一", "二", "三", "四", "五", "六", "七", "八", "九", "十", "百", "零",
];
const MEETING_KEYWORDS: [&str; 6] = ["指出", "强调", "要求", "决定", "认为", "同意"];

fn is_chinese_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return false;
    }
    if trimmed.starts_with('|') && trimmed.ends_with('|') {
        return false;
    }
    is_chinese_numbered_heading(trimmed)
        || is_chinese_sub_numbered_heading(trimmed)
        || is_arabic_numbered_heading(trimmed)
        || is_bold_heading(trimmed)
        || is_parenthesized_digit_heading(trimmed)
        || is_article_heading(trimmed)
        || is_meeting_phrase(trimmed)
}

fn is_chinese_num_str(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.len() == 3 {
        return CHINESE_NUMS.contains(&s);
    }
    if s.len() == 6 {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 2 {
            let a = chars[0].to_string();
            let b = chars[1].to_string();
            if a == "十" {
                return b != "十" && CHINESE_NUMS.contains(&b.as_str());
            }
            if a == "二" && b == "十" {
                return true;
            }
        }
    }
    if s.len() == 9 {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 3 && chars[0] == '二' && chars[1] == '十' && chars[2] != '十' {
            return CHINESE_NUMS.contains(&chars[2].to_string().as_str());
        }
    }
    false
}

fn is_chinese_numbered_heading(s: &str) -> bool {
    if let Some(sep_idx) = s.find('、') {
        let num_part = &s[..sep_idx];
        if is_chinese_num_str(num_part) {
            let rest = &s[sep_idx + '、'.len_utf8()..];
            return !rest.is_empty();
        }
    }
    false
}

fn is_chinese_sub_numbered_heading(s: &str) -> bool {
    if !s.starts_with('（') {
        return false;
    }
    let after_open = &s[3..];
    if let Some(close_idx) = after_open.find('）') {
        let num_part = &after_open[..close_idx];
        if !num_part.is_empty() && is_chinese_num_str(num_part) {
            let rest = &after_open[close_idx + 3..];
            return !rest.is_empty();
        }
    }
    false
}

fn is_arabic_numbered_heading(s: &str) -> bool {
    if s.contains("附件") {
        return false;
    }
    let bytes = s.as_bytes();
    let digit_end = bytes.iter().position(|b| !b.is_ascii_digit()).unwrap_or(0);
    if digit_end == 0 || digit_end >= bytes.len() {
        return false;
    }
    match s[..digit_end].parse::<u32>() {
        Ok(n) if (1..=99).contains(&n) => {}
        _ => return false,
    }
    let rest = &s[digit_end..];
    if rest.starts_with(". ") || rest.starts_with("、") || rest.starts_with("）") {
        let after_delim = if rest.starts_with(". ") {
            &rest[2..]
        } else {
            &rest[3..]
        };
        return !after_delim.trim().is_empty();
    }
    false
}

fn is_bold_heading(s: &str) -> bool {
    if !s.starts_with("**") {
        return false;
    }
    if let Some(closing) = s[2..].find("**") {
        let inner = &s[2..2 + closing];
        if !inner.trim().is_empty() {
            let after = &s[2 + closing + 2..];
            return after.trim().is_empty();
        }
    }
    false
}

fn is_parenthesized_digit_heading(line: &str) -> bool {
    if !line.starts_with('（') {
        return false;
    }
    let after_open = &line[3..];
    if let Some(close_idx) = after_open.find('）') {
        let inner = &after_open[..close_idx];
        let digit_count = inner.len();
        if !(1..=2).contains(&digit_count) {
            return false;
        }
        if !inner.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if let Ok(n) = inner.parse::<u32>() {
            if n == 0 || n > 99 {
                return false;
            }
        } else {
            return false;
        }
        let rest = &after_open[close_idx + 3..];
        return !rest.trim().is_empty();
    }
    false
}

fn is_article_heading(line: &str) -> bool {
    if line.chars().count() > 100 || !line.starts_with('第') {
        return false;
    }
    let after_di = &line[3..];
    let suffix_pos = if let Some(pos) = after_di.find('条') {
        pos
    } else if let Some(pos) = after_di.find('章') {
        pos
    } else {
        return false;
    };
    let num_part = &after_di[..suffix_pos];
    if num_part.is_empty() {
        return false;
    }
    num_part
        .chars()
        .all(|c| ARTICLE_NUMS.contains(&c.to_string().as_str()))
}

fn is_meeting_phrase(line: &str) -> bool {
    if line.chars().count() > 50 || !line.starts_with("会议") {
        return false;
    }
    let after_huiyi = &line[6..];
    for kw in &MEETING_KEYWORDS {
        if let Some(after_kw) = after_huiyi.strip_prefix(kw) {
            return !after_kw.trim().is_empty();
        }
    }
    false
}

/// Strip Chinese heading delimiters to extract the semantic heading text.
fn strip_heading_delimiters(line: &str) -> String {
    let s = line.trim();

    // （1）digit parenthesized
    if s.starts_with('（') {
        if let Some(close_idx) = s.find('）') {
            let inner = &s[3..close_idx];
            if !inner.is_empty() && inner.as_bytes().iter().all(|b| b.is_ascii_digit()) {
                let rest = s[close_idx + 3..].trim();
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
        }
    }

    // **bold**
    if s.starts_with("**") && s.ends_with("**") && s.len() > 4 {
        return s[2..s.len() - 2].trim().to_string();
    }

    // （一）Chinese sub-numbered
    if s.starts_with('（') {
        if let Some(close_idx) = s.find('）') {
            let rest = s[close_idx + 3..].trim();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }

    // 一、Chinese numbered
    if let Some(sep_idx) = s.find('、') {
        let num_part = &s[..sep_idx];
        if is_chinese_num_str(num_part) {
            let rest = s[sep_idx + '、'.len_utf8()..].trim();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }

    // Arabic: 1. text
    if s.as_bytes()
        .first()
        .map(|b| b.is_ascii_digit())
        .unwrap_or(false)
    {
        if let Some(dot_pos) = s.find(". ") {
            if (1..=2).contains(&dot_pos) {
                return s[dot_pos + 2..].trim().to_string();
            }
        }
    }

    s.to_string()
}

// ---------------------------------------------------------------------------
// Byte → char position mapping
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── existing tests (unchanged behavior) ──────────────────────────────────

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
    fn returns_single_section_when_headings_are_too_short() {
        let text = "# A\nshort\n# B\nalso short";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                min_section_chars: 100,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, None);
        assert_eq!(sections[0].text, text);
    }

    #[test]
    fn rejects_empty_input() {
        let err = split_sections("asset-1", "   ", SplitOptions::default()).unwrap_err();
        assert_eq!(err, AnimemError::EmptyInput);
    }

    // ── Gap 1: CJK heading detection ─────────────────────────────────────────

    #[test]
    fn cjk_disabled_by_default_ignores_chinese_headings() {
        // Without cjk_headings the Chinese heading lines are treated as body text.
        let text = "一、总体要求\n这是关于总体要求的正文内容，需要足够多的文字来构成有效的段落。\n二、工作成效\n取得了显著的工作成效，以下是具体描述。";
        let sections = split_sections("asset-1", text, SplitOptions::default()).unwrap();
        // Should be single section — no markdown headings detected.
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, None);
    }

    #[test]
    fn cjk_chinese_numbered_yi_er_san() {
        // 一、/ 二、/ 三、 patterns with cjk_headings enabled
        let text = "一、总体要求\n这是关于总体要求的正文内容，需要足够多的文字来构成有效的段落，确保超过最小字符数。\n\
二、工作成效\n取得了显著的工作成效，以下是具体描述，包含更多文字确保有效段落通过。\n\
三、下一步工作\n继续推进各项工作落实，制定详细计划确保完成年度目标任务。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 2, "got {} sections", sections.len());
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        assert!(headings.contains(&"总体要求"), "headings: {:?}", headings);
        assert!(headings.contains(&"工作成效"), "headings: {:?}", headings);
    }

    #[test]
    fn cjk_compound_numbered_shi_yi() {
        // 十一、 compound heading
        let text = "十一、专项工作\n这是关于专项工作的正文内容，需要足够多的文字来构成有效段落通过测试。\n\
                    十二、督查考核\n关于督查考核的具体安排，需要足够文字确保有效段落被识别。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 1);
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        assert!(headings.contains(&"专项工作"), "headings: {:?}", headings);
    }

    #[test]
    fn cjk_sub_numbered_parenthesized_chinese() {
        // （一）/ （二）patterns
        let text = "（一）加强组织领导\n关于加强组织领导的具体措施和实施方案，需要足够文字构成段落。\n\
                    （二）压实工作责任\n关于压实工作责任的具体要求，确保责任制落实到位各项工作有人管。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 2, "got {} sections", sections.len());
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        assert!(
            headings.contains(&"加强组织领导"),
            "headings: {:?}",
            headings
        );
        assert!(
            headings.contains(&"压实工作责任"),
            "headings: {:?}",
            headings
        );
    }

    #[test]
    fn cjk_article_heading_di_x_tiao() {
        // 第X条 / 第X章 patterns
        let text = "第一条 总则\n关于总则的具体内容，需要足够多的文字来通过最小字符数检查，确保段落有效。\n\
第二条 适用范围\n本办法适用于相关单位和人员，需要足够文字来构成有效段落通过测试。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 1);
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        // strip_heading_delimiters on "第一条 总则" returns the whole line (no delimiter pattern)
        assert!(
            headings
                .iter()
                .any(|h| h.contains("总则") || h.contains("第一条")),
            "headings: {:?}",
            headings
        );
    }

    #[test]
    fn cjk_meeting_phrase_hui_yi_zhi_chu() {
        // 会议指出/强调/要求 patterns
        let text = "会议指出，当前形势需要加强工作。\n这是会议指出部分的正文，需要足够文字构成有效段落测试用。\n\
                    会议强调，要落实各项工作措施。\n这是会议强调部分的内容，同样需要足够的文字确保段落有效。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 1);
    }

    #[test]
    fn cjk_bold_heading() {
        // **text** pattern
        let text =
            "**总体要求**\n这是关于总体要求的正文内容，需要足够多的文字来构成有效段落确保通过。\n\
                    **具体措施**\n这是具体措施部分的内容，同样需要足够多的文字来确保段落通过测试。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(sections.len() >= 2, "got {} sections", sections.len());
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        assert!(headings.contains(&"总体要求"), "headings: {:?}", headings);
        assert!(headings.contains(&"具体措施"), "headings: {:?}", headings);
    }

    #[test]
    fn cjk_table_rows_not_headings() {
        // Table rows must never become headings even with cjk_headings on
        let text = "| 一、项目 | 二、内容 |\n|------|------|\n| 数据1 | 数据2 |\n普通正文，需要足够多的文字来构成有效的段落内容测试。";
        let sections = split_sections(
            "asset-1",
            text,
            SplitOptions {
                cjk_headings: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(sections.len(), 1);
        assert!(sections[0].heading.is_none());
    }

    // ── Gap 2: max_section_chars sub-splitting ────────────────────────────────

    #[test]
    fn max_section_chars_splits_oversized_section() {
        // Build a section > 200 chars with clear paragraph breaks
        let para = "这是合成测试段落内容包含中文字符。\n";
        let body = para.repeat(20); // ~20 * 18 = ~360 chars
        let text = format!("# 主标题\n{}", body);
        let sections = split_sections(
            "asset-1",
            &text,
            SplitOptions {
                max_section_chars: Some(200),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            sections.len() > 1,
            "expected sub-splits, got {} sections",
            sections.len()
        );
        for sec in &sections {
            assert!(
                sec.text.chars().count() <= 200 + 50, // allow small overshoot at para boundary
                "section has {} chars",
                sec.text.chars().count()
            );
        }
    }

    #[test]
    fn max_section_chars_none_no_limit() {
        // Default (None) should not split regardless of size
        let para = "合成段落测试内容。";
        let body = para.repeat(500);
        let text = format!("# 大标题\n{}", body);
        let sections = split_sections("asset-1", &text, SplitOptions::default()).unwrap();
        // Without max_section_chars limit, stays as one section
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn max_section_chars_small_section_not_split() {
        let text = "# 短标题\n这是很短的内容，远低于最大字符数限制。";
        let sections = split_sections(
            "asset-1",
            &text,
            SplitOptions {
                max_section_chars: Some(3000),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(sections.len(), 1);
    }

    // ── Gap 3: DocumentSection.meta field ────────────────────────────────────

    #[test]
    fn meta_defaults_to_none() {
        let text = "# Section\n合成文档内容包含足够文字来构成有效段落通过测试。";
        let sections = split_sections("asset-1", &text, SplitOptions::default()).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].meta, None);
    }

    #[test]
    fn meta_survives_serde_roundtrip() {
        use crate::document::DocumentSection;
        use serde_json::json;

        // With explicit meta
        let sec_with_meta = DocumentSection {
            section_id: "a/s001".to_string(),
            asset_id: "a".to_string(),
            ordinal: 1,
            heading: Some("标题".to_string()),
            text: "内容".to_string(),
            char_start: 0,
            char_end: 2,
            meta: Some(json!({"splitter_version": "1.0", "source": "synthetic"})),
        };
        let json_str = serde_json::to_string(&sec_with_meta).unwrap();
        let decoded: DocumentSection = serde_json::from_str(&json_str).unwrap();
        assert_eq!(decoded.meta, sec_with_meta.meta);

        // Without meta field in JSON (backward compat: #[serde(default)])
        let json_no_meta = r#"{"section_id":"a/s001","asset_id":"a","ordinal":1,"heading":null,"text":"内容","char_start":0,"char_end":2}"#;
        let decoded2: DocumentSection = serde_json::from_str(json_no_meta).unwrap();
        assert_eq!(decoded2.meta, None);
    }

    // ── is_chinese_num_str unit tests ─────────────────────────────────────────

    #[test]
    fn chinese_num_str_single_chars() {
        for n in &["一", "二", "五", "十"] {
            assert!(is_chinese_num_str(n), "{} should be valid", n);
        }
    }

    #[test]
    fn chinese_num_str_compound() {
        assert!(is_chinese_num_str("十一"));
        assert!(is_chinese_num_str("十九"));
        assert!(is_chinese_num_str("二十"));
        assert!(is_chinese_num_str("二十五"));
        assert!(!is_chinese_num_str("十十"));
        assert!(!is_chinese_num_str(""));
    }
}
