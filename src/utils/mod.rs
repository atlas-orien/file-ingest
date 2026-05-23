mod file;

pub use file::{compute_sha256, extract_metadata};

use crate::model::{FileData, IngestOptions};

/// 附加文本内容到结果，并生成摘录
pub fn attach_text(result: &mut FileData, text: String, options: &IngestOptions) {
    let limit = options.max_text_length.unwrap_or(usize::MAX);
    let excerpt = truncate_text(&text, Some(limit));

    result.text = Some(text);
    result.text_excerpt = Some(excerpt);
}

/// 截断文本（如果超过最大长度）
pub fn truncate_text(text: &str, max_length: Option<usize>) -> String {
    match max_length {
        Some(max) if text.len() > max => {
            let mut truncated = String::new();
            for (count, ch) in text.chars().enumerate() {
                if count >= max {
                    truncated.push('…');
                    break;
                }
                truncated.push(ch);
            }
            truncated
        }
        _ => text.to_string(),
    }
}

/// 将文件名中的非法字符替换为 `_`
pub fn sanitize_filename(input: &str) -> String {
    let mut sanitized = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() {
        "asset".to_string()
    } else {
        sanitized
    }
}

/// 规范化中日韩文字之间的空格（主要用于 PDF 文本提取）
pub fn normalize_cjk_spacing(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(text.len());
    for i in 0..chars.len() {
        let ch = chars[i];
        if ch == ' ' {
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let next = if i + 1 < chars.len() {
                Some(chars[i + 1])
            } else {
                None
            };

            if let (Some(p), Some(n)) = (prev, next) {
                let prev_cjk = is_cjk_or_punct(p);
                let next_cjk = is_cjk_or_punct(n);
                let prev_latin = is_latin_alnum(p);
                let next_latin = is_latin_alnum(n);

                if (prev_cjk && next_cjk) || (prev_cjk && next_latin) || (prev_latin && next_cjk) {
                    continue;
                }
            }
        }
        out.push(ch);
    }

    out
}

fn is_latin_alnum(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

fn is_cjk_or_punct(ch: char) -> bool {
    is_cjk(ch) || is_cjk_punct(ch)
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x3400..=0x4DBF // CJK Extension A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0x20000..=0x2A6DF // CJK Extension B
        | 0x2A700..=0x2B73F // CJK Extension C
        | 0x2B740..=0x2B81F // CJK Extension D
        | 0x2B820..=0x2CEAF // CJK Extension E
        | 0x2CEB0..=0x2EBEF // CJK Extension F
        | 0x2F800..=0x2FA1F // CJK Compatibility Ideographs Supplement
    )
}

fn is_cjk_punct(ch: char) -> bool {
    matches!(ch as u32,
        0x3000..=0x303F // CJK Symbols and Punctuation
        | 0xFF00..=0xFFEF // Halfwidth and Fullwidth Forms (punctuation)
    )
}
