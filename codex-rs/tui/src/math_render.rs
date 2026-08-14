//! LaTeX math rendering for terminal transcript lines.
//!
//! Markdown parsing stays responsible for identifying math spans. This module only converts the
//! TeX payload into bounded Unicode character art and provides a compact source fallback for
//! unsupported, empty, or oversized output.

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

/// Returns a compact, delimiter-free source representation when Unicode rendering is unavailable.
///
/// Display math often arrives with one TeX token per source line. Emitting that source verbatim
/// makes a single failed formula look like a large broken paragraph, so the fallback deliberately
/// folds whitespace while retaining every non-presentation command for diagnosis and copy/paste.
pub(crate) fn math_source_fallback(latex: &str) -> String {
    normalize_for_terminal(latex).into_owned()
}

fn render_bounded(latex: &str, max_width: Option<usize>) -> Option<term_maths::RenderedBlock> {
    if latex.is_empty() || latex.len() > MAX_LATEX_BYTES {
        return None;
    }
    let (latex, draw_box) = strip_outer_box(latex).map_or((latex, false), |inner| (inner, true));
    let latex = normalize_for_terminal(latex);
    if latex.len() > MAX_LATEX_BYTES {
        return None;
    }
    let mut block = catch_unwind(AssertUnwindSafe(|| term_maths::render(&latex))).ok()?;
    if draw_box {
        block = add_terminal_box(&block);
    }
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

/// Rewrites common presentation-level LaTeX commands and folds source whitespace for
/// `term-maths`. Command names are scanned as tokens so, for example, `\bm` never rewrites a
/// longer command with the same prefix. Unsupported commands still reach the existing source
/// fallback instead of being silently discarded.
fn normalize_for_terminal(latex: &str) -> Cow<'_, str> {
    let commands = normalize_commands_for_terminal(latex);
    collapse_source_whitespace(commands)
}

fn collapse_source_whitespace(commands: Cow<'_, str>) -> Cow<'_, str> {
    let mut normalized = String::with_capacity(commands.len());
    let mut pending_space = false;

    for character in commands.chars() {
        if character.is_whitespace() {
            pending_space = !normalized.is_empty();
        } else {
            if pending_space {
                normalized.push(' ');
                pending_space = false;
            }
            normalized.push(character);
        }
    }

    if normalized == commands {
        commands
    } else {
        Cow::Owned(normalized)
    }
}

fn normalize_commands_for_terminal(latex: &str) -> Cow<'_, str> {
    let bytes = latex.as_bytes();
    let mut normalized = String::with_capacity(latex.len());
    let mut copied_until = 0;
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\\' {
            cursor += 1;
            continue;
        }
        if let Some((matched_len, replacement)) = terminal_sequence_replacement(&latex[cursor..]) {
            normalized.push_str(&latex[copied_until..cursor]);
            normalized.push_str(replacement);
            cursor += matched_len;
            copied_until = cursor;
            continue;
        }
        if cursor + 1 >= bytes.len() || !bytes[cursor + 1].is_ascii_alphabetic() {
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
            "textbf" | "textit" | "texttt" | "emph" => Some(r"\text"),
            "top" => Some(r"\mathsf{T}"),
            "colon" => Some(":"),
            "vert" | "lvert" | "rvert" => Some("|"),
            "Vert" | "lVert" | "rVert" => Some("‖"),
            "longrightarrow" => Some(r"\rightarrow"),
            "Longrightarrow" => Some(r"\Rightarrow"),
            "longleftarrow" => Some(r"\leftarrow"),
            "Longleftarrow" => Some(r"\Leftarrow"),
            "longleftrightarrow" => Some(r"\leftrightarrow"),
            "Longleftrightarrow" => Some(r"\Leftrightarrow"),
            "boxed" => Some(""),
            "big" | "bigl" | "bigr" | "bigm" | "Big" | "Bigl" | "Bigr" | "Bigm" | "bigg"
            | "biggl" | "biggr" | "biggm" | "Bigg" | "Biggl" | "Biggr" | "Biggm" | "middle" => {
                Some("")
            }
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

/// `rust-latex-parser`, used by `term-maths`, accepts a delimiter character directly after
/// `\left`/`\right`, while standard TeX escapes braces and often names vertical bars. Translate
/// only that narrow syntax gap before the normal command pass.
fn terminal_sequence_replacement(input: &str) -> Option<(usize, &'static str)> {
    const REPLACEMENTS: &[(&str, &str)] = &[
        (r"\left\lVert", "\\left‖"),
        (r"\right\rVert", "\\right‖"),
        (r"\left\Vert", "\\left‖"),
        (r"\right\Vert", "\\right‖"),
        (r"\left\lvert", r"\left|"),
        (r"\right\rvert", r"\right|"),
        (r"\left\vert", r"\left|"),
        (r"\right\vert", r"\right|"),
        (r"\left\{", r"\left{"),
        (r"\right\}", r"\right}"),
        (r"\left\}", r"\left}"),
        (r"\right\{", r"\right{"),
    ];

    REPLACEMENTS.iter().find_map(|(source, replacement)| {
        input
            .starts_with(source)
            .then_some((source.len(), *replacement))
    })
}

fn strip_outer_box(latex: &str) -> Option<&str> {
    const PREFIX: &str = r"\boxed{";
    let trimmed = latex.trim();
    let rest = trimmed.strip_prefix(PREFIX)?;
    let mut depth = 1_usize;
    let mut escaped = false;

    for (offset, character) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return rest[offset + character.len_utf8()..]
                        .trim()
                        .is_empty()
                        .then_some(&rest[..offset]);
                }
            }
            _ => {}
        }
    }
    None
}

fn add_terminal_box(block: &term_maths::RenderedBlock) -> term_maths::RenderedBlock {
    let mut rows = Vec::with_capacity(block.height() + 2);
    let mut top = Vec::with_capacity(block.width() + 2);
    top.push("┌".to_string());
    top.extend(std::iter::repeat_n("─".to_string(), block.width()));
    top.push("┐".to_string());
    rows.push(top);

    for row in block.cells() {
        let mut boxed_row = Vec::with_capacity(row.len() + 2);
        boxed_row.push("│".to_string());
        boxed_row.extend(row.iter().cloned());
        boxed_row.push("│".to_string());
        rows.push(boxed_row);
    }

    let mut bottom = Vec::with_capacity(block.width() + 2);
    bottom.push("└".to_string());
    bottom.extend(std::iter::repeat_n("─".to_string(), block.width()));
    bottom.push("┘".to_string());
    rows.push(bottom);

    term_maths::RenderedBlock::new(rows, block.baseline() + 1)
}

#[cfg(test)]
#[path = "math_render_tests.rs"]
mod tests;
