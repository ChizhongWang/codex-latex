//! Compatibility normalization for LaTeX-style math delimiters.
//!
//! `pulldown-cmark` recognizes `$...$` and `$$...$$`, while model output commonly also uses
//! `\(...\)` and `\[...\]`. Replacements here deliberately preserve byte length so parser event
//! offsets still index the original Markdown source used for fallback rendering and streaming.

use pulldown_cmark::Event;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use std::borrow::Cow;
use std::ops::Range;

#[derive(Clone, Copy)]
enum DelimiterKind {
    Inline,
    Display,
}

pub(super) fn normalize(input: &str, options: Options) -> Cow<'_, str> {
    if !input.contains("\\(") && !input.contains("\\[") && !input.contains("$$") {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut candidates = Vec::new();
    let mut excluded_ranges = Vec::<Range<usize>>::new();
    let mut in_code_block = false;
    let mut code_block_start = None;
    for (event, range) in Parser::new_ext(input, options).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                code_block_start = Some(range.start);
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if let Some(start) = code_block_start.take() {
                    excluded_ranges.push(start..range.end);
                }
            }
            Event::Code(_) => excluded_ranges.push(range),
            Event::Text(_) if !in_code_block => {
                for position in range {
                    let marker = bytes[position];
                    if matches!(marker, b'(' | b')' | b'[' | b']')
                        && has_single_backslash_before(bytes, position)
                    {
                        candidates.push((position - 1, marker));
                    }
                }
            }
            Event::Start(_)
            | Event::End(_)
            | Event::Text(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule
            | Event::TaskListMarker(_) => {}
        }
    }
    if let Some(start) = code_block_start {
        excluded_ranges.push(start..input.len());
    }

    let mut open = None;
    let mut normalized = None;
    for (position, marker) in candidates {
        match (open, marker) {
            (None, b'(') => open = Some((position, DelimiterKind::Inline)),
            (None, b'[') => open = Some((position, DelimiterKind::Display)),
            (Some((open_position, DelimiterKind::Inline)), b')') => {
                let output = normalized.get_or_insert_with(|| bytes.to_vec());
                output[open_position] = b'$';
                output[open_position + 1] = b'{';
                output[position] = b'}';
                output[position + 1] = b'$';
                open = None;
            }
            (Some((open_position, DelimiterKind::Display)), b']') => {
                let output = normalized.get_or_insert_with(|| bytes.to_vec());
                output[open_position] = b'$';
                output[open_position + 1] = b'$';
                output[position] = b'$';
                output[position + 1] = b'$';
                open = None;
            }
            _ => {}
        }
    }

    flatten_display_math_lines(bytes, &excluded_ranges, &mut normalized);

    match normalized.map(String::from_utf8) {
        Some(Ok(output)) => Cow::Owned(output),
        Some(Err(_)) | None => Cow::Borrowed(input),
    }
}

/// CommonMark can interpret a standalone `=` inside a multi-line `$$` block as a Setext heading
/// underline before its math extension gets a chance to claim the block. Folding only CR/LF bytes
/// inside paired display delimiters removes that ambiguity and preserves every source offset.
fn flatten_display_math_lines(
    original: &[u8],
    excluded_ranges: &[Range<usize>],
    normalized: &mut Option<Vec<u8>>,
) {
    let source = normalized.as_deref().unwrap_or(original);
    let mut delimiters = Vec::new();
    let mut cursor = 0;
    let mut excluded = 0;

    while cursor + 1 < source.len() {
        while excluded < excluded_ranges.len() && excluded_ranges[excluded].end <= cursor {
            excluded += 1;
        }
        if excluded_ranges
            .get(excluded)
            .is_some_and(|range| range.contains(&cursor))
        {
            cursor = excluded_ranges[excluded].end;
            continue;
        }

        if source[cursor..].starts_with(b"$$")
            && source.get(cursor.wrapping_sub(1)) != Some(&b'$')
            && source.get(cursor + 2) != Some(&b'$')
            && !is_escaped(source, cursor)
        {
            delimiters.push(cursor);
            cursor += 2;
        } else {
            cursor += 1;
        }
    }

    let multiline_contents = delimiters
        .chunks_exact(2)
        .map(|pair| pair[0] + 2..pair[1])
        .filter(|content| {
            source[content.clone()]
                .iter()
                .any(|byte| matches!(byte, b'\n' | b'\r'))
        })
        .collect::<Vec<_>>();

    for content in multiline_contents {
        let output = normalized.get_or_insert_with(|| original.to_vec());
        for byte in &mut output[content] {
            if matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
    }
}

fn is_escaped(bytes: &[u8], position: usize) -> bool {
    let mut backslashes = 0;
    let mut cursor = position;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

fn has_single_backslash_before(bytes: &[u8], position: usize) -> bool {
    position > 0 && bytes[position - 1] == b'\\' && (position < 2 || bytes[position - 2] != b'\\')
}

#[cfg(test)]
#[path = "latex_delimiters_tests.rs"]
mod tests;
