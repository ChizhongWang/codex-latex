//! LaTeX math rendering for terminal transcript lines.
//!
//! Markdown parsing stays responsible for identifying math spans. This module only converts the
//! TeX payload into bounded Unicode character art. Callers retain the original Markdown source so
//! unsupported, empty, or oversized output can fall back without losing content.

use std::borrow::Cow;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

const MAX_LATEX_BYTES: usize = 16 * 1024;
const MAX_RENDERED_HEIGHT: usize = 24;

pub(crate) fn render_inline_math(latex: &str, max_width: Option<usize>) -> Option<String> {
    let block = render_bounded(latex, max_width)?;
    if block.height() != 1 {
        return None;
    }
    let rendered = block.to_string();
    (!rendered.is_empty() && !rendered.contains('\n')).then(|| rendered.trim_end().to_string())
}

pub(crate) fn render_display_math(latex: &str, max_width: Option<usize>) -> Option<Vec<String>> {
    let block = render_bounded(latex.trim(), max_width)?;
    let lines = block
        .to_string()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    (!lines.is_empty() && lines.iter().any(|line| !line.is_empty())).then_some(lines)
}

fn render_bounded(latex: &str, max_width: Option<usize>) -> Option<term_maths::RenderedBlock> {
    if latex.is_empty() || latex.len() > MAX_LATEX_BYTES {
        return None;
    }
    let latex = normalize_for_terminal(latex);
    if latex.len() > MAX_LATEX_BYTES {
        return None;
    }
    let block = catch_unwind(AssertUnwindSafe(|| term_maths::render(&latex))).ok()?;
    let rendered = block.to_string();
    if block.is_empty()
        || block.height() > MAX_RENDERED_HEIGHT
        || max_width.is_some_and(|max_width| block.width() > max_width)
        || rendered.contains('\\')
    {
        return None;
    }
    Some(block)
}

/// Rewrites common presentation-level LaTeX commands to equivalents understood by
/// `term-maths`. Command names are scanned as tokens so, for example, `\bm` never rewrites a
/// longer command with the same prefix. Unsupported commands still reach the existing source
/// fallback instead of being silently discarded.
fn normalize_for_terminal(latex: &str) -> Cow<'_, str> {
    let bytes = latex.as_bytes();
    let mut normalized = String::with_capacity(latex.len());
    let mut copied_until = 0;
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\\'
            || cursor + 1 >= bytes.len()
            || !bytes[cursor + 1].is_ascii_alphabetic()
        {
            cursor += 1;
            continue;
        }

        let command_start = cursor;
        cursor += 2;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let command = &latex[command_start + 1..cursor];
        let replacement = match command {
            "operatorname" => Some(r"\mathrm"),
            "dfrac" | "tfrac" => Some(r"\frac"),
            "boldsymbol" | "bm" => Some(r"\mathbf"),
            "textrm" | "textnormal" => Some(r"\mathrm"),
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" | "limits"
            | "nolimits" => Some(""),
            _ => None,
        };

        if let Some(replacement) = replacement {
            normalized.push_str(&latex[copied_until..command_start]);
            normalized.push_str(replacement);
            if command == "operatorname" && bytes.get(cursor) == Some(&b'*') {
                cursor += 1;
            }
            copied_until = cursor;
        }
    }

    if copied_until == 0 {
        Cow::Borrowed(latex)
    } else {
        normalized.push_str(&latex[copied_until..]);
        Cow::Owned(normalized)
    }
}

#[cfg(test)]
#[path = "math_render_tests.rs"]
mod tests;
