use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use time::{OffsetDateTime, format_description};

/// 规范化选项：每条规则都可以独立开关，保证可复现、可回退。
#[derive(Debug, Clone)]
pub struct NormalizationOptions {
    /// 启用页眉/页脚/页码清理
    pub enable_header_footer_cleanup: bool,
    /// 启用页边界标记插入
    pub enable_page_markers: bool,
    /// 启用断行合并（段落重建）
    pub enable_line_merge: bool,
    /// 启用标题候选规范化
    pub enable_heading_normalize: bool,
    /// 将被移除内容用 HTML 注释标记，而不是直接删除
    pub keep_removed_as_comment: bool,
    /// 扫描页眉/页脚的行数
    pub header_footer_scan_lines: usize,
    /// 被认为重复的最小页数
    pub header_footer_min_repeat: usize,
    /// 参与页眉/页脚统计的最大行长度
    pub max_header_footer_line_len: usize,
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            enable_header_footer_cleanup: true,
            enable_page_markers: true,
            enable_line_merge: true,
            enable_heading_normalize: true,
            keep_removed_as_comment: true,
            header_footer_scan_lines: 2,
            header_footer_min_repeat: 2,
            max_header_footer_line_len: 40,
        }
    }
}

/// 将 raw Markdown 规范化为 canonical Markdown。
pub fn normalize_markdown(raw: &str, options: &NormalizationOptions) -> String {
    let timestamp = OffsetDateTime::now_utc();
    normalize_markdown_with_timestamp(raw, options, timestamp)
}

/// 提供可控时间戳的规范化入口，便于测试与可重复执行。
pub fn normalize_markdown_with_timestamp(
    raw: &str,
    options: &NormalizationOptions,
    timestamp: OffsetDateTime,
) -> String {
    let raw_body = strip_frontmatter(raw);
    let pages = split_pages(raw_body);
    let header_footer_map = if options.enable_header_footer_cleanup {
        build_header_footer_map(&pages, options)
    } else {
        HeaderFooterMap::default()
    };

    let mut normalized_pages = Vec::with_capacity(pages.len());
    for (idx, page) in pages.iter().enumerate() {
        let page_no = page.number.unwrap_or(idx + 1);
        let mut lines = page.lines.clone();

        if options.enable_header_footer_cleanup {
            lines = remove_header_footer_lines(
                &lines,
                page_no,
                &header_footer_map,
                options,
            );
        }

        if options.enable_line_merge {
            lines = normalize_blank_lines(&lines);
            lines = merge_lines(&lines);
            lines = split_page_reference_lines(&lines);
        }

        if options.enable_heading_normalize {
            lines = normalize_headings(&lines);
        }

        if options.enable_page_markers {
            let mut with_marker = Vec::with_capacity(lines.len() + 1);
            with_marker.push(format!("<!-- page: {page_no} -->"));
            with_marker.extend(lines);
            normalized_pages.push(with_marker);
        } else {
            normalized_pages.push(lines);
        }
    }

    let body = normalized_pages
        .iter()
        .map(|page| page.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n");

    let mut output = String::new();
    output.push_str("---\n");
    output.push_str("source: raw\n");
    output.push_str("normalized_at: ");
    output.push_str(&format_timestamp(timestamp));
    output.push_str("\n---\n\n");
    output.push_str(body.trim_end());
    output.push('\n');
    output
}

/// 从 raw.md 生成 canonical.md。
pub fn normalize_file<P: AsRef<Path>, Q: AsRef<Path>>(
    raw_path: P,
    canonical_path: Q,
    options: &NormalizationOptions,
) -> Result<()> {
    let raw = fs::read_to_string(raw_path)?;
    let normalized = normalize_markdown(&raw, options);
    fs::write(canonical_path, normalized)?;
    Ok(())
}

fn format_timestamp(ts: OffsetDateTime) -> String {
    let format = format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second]Z",
    )
    .unwrap_or_else(|_| format_description::parse("[year]-[month]-[day]").unwrap());
    ts.format(&format).unwrap_or_else(|_| ts.to_string())
}

fn split_pages(raw: &str) -> Vec<Page> {
    let mut result = Vec::new();
    let mut current = Page::new(None);

    for line in raw.lines() {
        if let Some(marker) = find_page_marker(line) {
            if !marker.prefix.trim().is_empty() {
                current.lines.push(marker.prefix.trim_end().to_string());
            }

            if !current.lines.is_empty() || current.number.is_some() {
                result.push(current);
            }

            current = Page::new(marker.page_no);
            if !marker.suffix.trim().is_empty() {
                current.lines.push(marker.suffix.trim_start().to_string());
            }
            continue;
        }

        current.lines.push(line.to_string());
    }

    if !current.lines.is_empty() || result.is_empty() {
        result.push(current);
    }

    result
}

#[derive(Clone)]
struct Page {
    number: Option<usize>,
    lines: Vec<String>,
}

impl Page {
    fn new(number: Option<usize>) -> Self {
        Self {
            number,
            lines: Vec::new(),
        }
    }
}

#[derive(Default)]
struct HeaderFooterMap {
    header_counts: HashMap<String, usize>,
    footer_counts: HashMap<String, usize>,
}

fn build_header_footer_map(
    pages: &[Page],
    options: &NormalizationOptions,
) -> HeaderFooterMap {
    let mut header_counts = HashMap::new();
    let mut footer_counts = HashMap::new();

    for page in pages {
        let header = take_boundary_lines(&page.lines, options.header_footer_scan_lines, true);
        for line in header {
            *header_counts.entry(line).or_insert(0) += 1;
        }
        let footer = take_boundary_lines(&page.lines, options.header_footer_scan_lines, false);
        for line in footer {
            *footer_counts.entry(line).or_insert(0) += 1;
        }
    }

    HeaderFooterMap {
        header_counts,
        footer_counts,
    }
}

fn take_boundary_lines(page: &[String], count: usize, from_top: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let iter: Box<dyn Iterator<Item = &String>> = if from_top {
        Box::new(page.iter())
    } else {
        Box::new(page.iter().rev())
    };

    for line in iter {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        lines.push(trimmed.to_string());
        if lines.len() >= count {
            break;
        }
    }
    lines
}

fn remove_header_footer_lines(
    lines: &[String],
    _page_no: usize,
    map: &HeaderFooterMap,
    options: &NormalizationOptions,
) -> Vec<String> {
    let mut cleaned = Vec::with_capacity(lines.len());
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            cleaned.push(line.clone());
            continue;
        }

        let is_short = trimmed.chars().count() <= options.max_header_footer_line_len;
        let is_repeated_header = is_short
            && map
                .header_counts
                .get(trimmed)
                .is_some_and(|c| *c >= options.header_footer_min_repeat);
        let is_repeated_footer = is_short
            && map
                .footer_counts
                .get(trimmed)
                .is_some_and(|c| *c >= options.header_footer_min_repeat);
        let is_page_number = is_page_number_line(trimmed);

        if is_repeated_header || is_repeated_footer || is_page_number {
            if options.keep_removed_as_comment {
                cleaned.push(format!("<!-- removed: {} -->", escape_comment(trimmed)));
            }
            continue;
        }

        let mut sanitized = strip_inline_page_marker(line);
        sanitized = strip_repeated_fragments(&sanitized, map, options);
        cleaned.push(sanitized);
    }
    cleaned
}

fn is_page_number_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("page ") && lower[5..].trim().chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let cleaned = trimmed
        .trim_matches('-')
        .trim_matches('—')
        .trim();
    cleaned.chars().all(|c| c.is_ascii_digit())
}

fn strip_inline_page_marker(line: &str) -> String {
    if let Some(marker) = find_page_marker(line) {
        let mut combined = String::new();
        combined.push_str(marker.prefix.trim_end());
        if !marker.suffix.trim().is_empty() {
            if !combined.is_empty() {
                combined.push(' ');
            }
            combined.push_str(marker.suffix.trim_start());
        }
        return combined.trim().to_string();
    }
    line.to_string()
}

fn merge_lines(lines: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let mut current = lines[i].clone();

        if current.trim().is_empty() {
            merged.push(current);
            i += 1;
            continue;
        }

        while i + 1 < lines.len() {
            let next = &lines[i + 1];
            if next.trim().is_empty() {
                break;
            }
            if is_heading_candidate(current.trim()) {
                break;
            }
            if is_heading_candidate(next.trim()) {
                break;
            }
            if !should_merge_line(&current, next) {
                break;
            }
            current = merge_two_lines(&current, next);
            i += 1;
        }

        merged.push(current);
        i += 1;
    }

    merged
}

fn normalize_blank_lines(lines: &[String]) -> Vec<String> {
    let mut normalized = Vec::with_capacity(lines.len());
    for i in 0..lines.len() {
        let line = &lines[i];
        if !line.trim().is_empty() {
            normalized.push(line.clone());
            continue;
        }

        let prev = normalized.last().map(|s| s.as_str()).unwrap_or("");
        let next = lines.get(i + 1).map(|s| s.as_str()).unwrap_or("");
        let prev_non_empty = !prev.trim().is_empty();
        let next_non_empty = !next.trim().is_empty();
        let prev_terminal = last_non_space_char(prev).is_some_and(is_terminal_punct);
        let prev_heading = is_heading_candidate(prev.trim());
        let next_heading = is_heading_candidate(next.trim());

        if prev_non_empty
            && next_non_empty
            && !prev_terminal
            && !prev_heading
            && !next_heading
        {
            continue;
        }

        if normalized.last().is_some_and(|last| last.trim().is_empty()) {
            continue;
        }

        normalized.push(String::new());
    }
    normalized
}

fn split_page_reference_lines(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let refs = find_page_ref_positions(line);
        if line.chars().count() < 80 || refs.len() < 2 {
            out.push(line.clone());
            continue;
        }
        let mut start = 0usize;
        for end in refs {
            let segment = line[start..end].trim();
            if !segment.is_empty() {
                out.push(segment.to_string());
            }
            start = end;
        }
        let tail = line[start..].trim();
        if !tail.is_empty() {
            out.push(tail.to_string());
        }
    }
    out
}

fn should_merge_line(current: &str, next: &str) -> bool {
    let current_trim = current.trim_end();
    let next_trim = next.trim_start();

    if current_trim.is_empty() || next_trim.is_empty() {
        return false;
    }

    let last = last_non_space_char(current_trim);
    if let Some(ch) = last {
        if is_terminal_punct(ch) {
            return false;
        }
    }

    let next_first = next_trim.chars().next();
    if let Some(ch) = next_first {
        if ch.is_ascii_uppercase() {
            return false;
        }
    }

    true
}

fn merge_two_lines(current: &str, next: &str) -> String {
    let current_trim = current.trim_end();
    let next_trim = next.trim_start();

    if current_trim.ends_with('-') && next_trim.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        let without_hyphen = current_trim.trim_end_matches('-');
        return format!("{}{}", without_hyphen, next_trim);
    }

    let last_char = last_non_space_char(current_trim);
    let next_char = next_trim.chars().next();
    let need_space = match (last_char, next_char) {
        (Some(a), Some(b)) => !(is_cjk(a) && is_cjk(b)),
        _ => true,
    };

    if need_space {
        format!("{} {}", current_trim, next_trim)
    } else {
        format!("{}{}", current_trim, next_trim)
    }
}

fn normalize_headings(lines: &[String]) -> Vec<String> {
    let mut normalized = Vec::with_capacity(lines.len());
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            normalized.push(line.clone());
            continue;
        }
        if trimmed.starts_with('#') {
            normalized.push(line.clone());
            continue;
        }
        if let Some(level) = classify_heading_level(trimmed) {
            let hashes = "#".repeat(level as usize);
            normalized.push(format!("{hashes} {trimmed}"));
        } else {
            normalized.push(line.clone());
        }
    }
    normalized
}

fn classify_heading_level(line: &str) -> Option<u8> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return Some(2);
    }
    if trimmed.chars().count() > 60 {
        return None;
    }

    if trimmed.contains("目录")
        || trimmed.to_ascii_uppercase().contains("CONTENTS")
    {
        return Some(2);
    }

    if is_chapter_heading(trimmed) {
        return Some(2);
    }
    if is_section_heading(trimmed) {
        return Some(3);
    }
    if let Some(level) = numeric_heading_level(trimmed) {
        return Some(level);
    }
    if trimmed.ends_with(':') || trimmed.ends_with('：') {
        return Some(3);
    }
    if is_short_title_like(trimmed) {
        return Some(3);
    }
    None
}

fn is_heading_candidate(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('#') || classify_heading_level(trimmed).is_some()
}

fn is_chapter_heading(line: &str) -> bool {
    line.starts_with('第') && (line.contains('章') || line.contains('卷') || line.contains('篇'))
}

fn is_section_heading(line: &str) -> bool {
    line.starts_with('第') && line.contains('节')
}

fn numeric_heading_level(line: &str) -> Option<u8> {
    let mut chars = line.chars().peekable();
    if !chars.peek().is_some_and(|c| c.is_ascii_digit()) {
        if line.starts_with('（') && line.ends_with('）') {
            return Some(3);
        }
        if is_chinese_number_heading(line) {
            return Some(3);
        }
        return None;
    }

    let mut dot_count = 0usize;
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            chars.next();
            continue;
        }
        if *ch == '.' {
            dot_count += 1;
            chars.next();
            continue;
        }
        if *ch == '、' || *ch == ')' {
            break;
        }
        break;
    }

    if dot_count == 0 {
        Some(2)
    } else {
        Some(3)
    }
}

fn is_chinese_number_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 2 {
        return false;
    }
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap_or(' ');
    let second = chars.next().unwrap_or(' ');
    is_chinese_number(first) && second == '、'
}

fn is_chinese_number(ch: char) -> bool {
    matches!(ch, '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百' | '千')
}

fn is_short_title_like(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.chars().count() > 12 {
        return false;
    }
    if trimmed.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if trimmed.contains(':') || trimmed.contains('：') || trimmed.contains('/') {
        return false;
    }
    if trimmed.chars().any(is_terminal_punct) {
        return false;
    }
    let alpha = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    let cjk = trimmed.chars().filter(|c| is_cjk(*c)).count();
    alpha + cjk >= 2
}

fn last_non_space_char(s: &str) -> Option<char> {
    s.chars().rev().find(|c| !c.is_whitespace())
}

fn is_terminal_punct(ch: char) -> bool {
    matches!(
        ch,
        '.' | '。' | '!' | '！' | '?' | '？' | ';' | '；' | '…'
    )
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xF900..=0xFAFF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2B73F
        | 0x2B740..=0x2B81F
        | 0x2B820..=0x2CEAF
        | 0x2CEB0..=0x2EBEF
        | 0x2F800..=0x2FA1F
    )
}

fn escape_comment(text: &str) -> String {
    text.replace("--", "- -")
}

fn strip_repeated_fragments(
    line: &str,
    map: &HeaderFooterMap,
    options: &NormalizationOptions,
) -> String {
    let mut output = line.to_string();
    let repeated = map
        .header_counts
        .iter()
        .chain(map.footer_counts.iter())
        .filter(|(_, count)| **count >= options.header_footer_min_repeat)
        .map(|(text, _)| text.as_str());

    for fragment in repeated {
        if !is_header_fragment_candidate(fragment) {
            continue;
        }
        if output.contains(fragment) {
            output = output.replace(fragment, "").trim().to_string();
        }
    }

    output
}

fn is_header_fragment_candidate(text: &str) -> bool {
    let len = text.chars().count();
    if len < 12 {
        return false;
    }
    text.chars().any(|c| c.is_ascii_digit()) || text.contains('|')
}

fn strip_frontmatter(raw: &str) -> &str {
    let mut lines = raw.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return raw;
    }

    let mut offset = 0usize;
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        offset += line.len() + 1;
        if idx > 0 && trimmed == "---" {
            break;
        }
    }

    raw.get(offset..).unwrap_or(raw)
}

struct PageMarker<'a> {
    page_no: Option<usize>,
    prefix: &'a str,
    suffix: &'a str,
}

fn find_page_marker(line: &str) -> Option<PageMarker<'_>> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    for i in 0..chars.len() {
        if chars[i].1 != '第' {
            continue;
        }
        let mut j = i + 1;
        let mut digits = String::new();
        while j < chars.len() && chars[j].1.is_ascii_digit() {
            digits.push(chars[j].1);
            j += 1;
        }
        if digits.is_empty() {
            continue;
        }
        if j >= chars.len() || chars[j].1 != '页' {
            continue;
        }
        j += 1;
        let page_no = digits.parse::<usize>().ok();
        let start = chars[i].0;
        let mut end = chars[j - 1].0 + chars[j - 1].1.len_utf8();
        let mut has_total = false;
        if j < chars.len() && chars[j].1 == '共' {
            let mut k = j + 1;
            let mut total_digits = String::new();
            while k < chars.len() && chars[k].1.is_ascii_digit() {
                total_digits.push(chars[k].1);
                k += 1;
            }
            if !total_digits.is_empty() && k < chars.len() && chars[k].1 == '页' {
                end = chars[k].0 + chars[k].1.len_utf8();
                has_total = true;
            }
        }

        let prefix = &line[..start];
        let suffix = &line[end..];
        let trimmed = line.trim();
        let marker_only = trimmed.len() == (end - start)
            || (prefix.trim().is_empty() && suffix.trim().is_empty());
        if has_total || marker_only {
            return Some(PageMarker {
                page_no,
                prefix,
                suffix,
            });
        }
    }
    None
}

fn find_page_ref_positions(line: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].1 != '第' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        let mut digits = String::new();
        while j < chars.len() && chars[j].1.is_ascii_digit() {
            digits.push(chars[j].1);
            j += 1;
        }
        if digits.is_empty() {
            i += 1;
            continue;
        }
        if j < chars.len() && chars[j].1 == '页' {
            let end = chars[j].0 + chars[j].1.len_utf8();
            positions.push(end);
            i = j + 1;
            continue;
        }
        i += 1;
    }
    positions
}
