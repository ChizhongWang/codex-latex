//! LaTeX math rendering for terminal transcript lines.
//!
//! Markdown parsing stays responsible for identifying math spans. This module only converts the
//! TeX payload into bounded Unicode character art. Callers retain the original Markdown source so
//! unsupported, empty, or oversized output can fall back without losing content.

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
    let block = catch_unwind(AssertUnwindSafe(|| term_maths::render(latex))).ok()?;
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

#[cfg(test)]
#[path = "math_render_tests.rs"]
mod tests;
