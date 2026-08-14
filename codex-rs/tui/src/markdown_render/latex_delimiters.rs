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

#[derive(Clone, Copy)]
enum DelimiterKind {
    Inline,
    Display,
}

pub(super) fn normalize(input: &str, options: Options) -> Cow<'_, str> {
    if !input.contains("\\(") && !input.contains("\\[") {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut candidates = Vec::new();
    let mut in_code_block = false;
    for (event, range) in Parser::new_ext(input, options).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(TagEnd::CodeBlock) => in_code_block = false,
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
            | Event::Code(_)
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

    match normalized.map(String::from_utf8) {
        Some(Ok(output)) => Cow::Owned(output),
        Some(Err(_)) | None => Cow::Borrowed(input),
    }
}

fn has_single_backslash_before(bytes: &[u8], position: usize) -> bool {
    position > 0 && bytes[position - 1] == b'\\' && (position < 2 || bytes[position - 2] != b'\\')
}

#[cfg(test)]
#[path = "latex_delimiters_tests.rs"]
mod tests;
