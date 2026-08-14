use super::normalize;
use pretty_assertions::assert_eq;
use pulldown_cmark::Options;

#[test]
fn normalizes_matched_inline_and_display_delimiters_without_changing_length() {
    let input = r"Inline \(x^2\), then \[\frac{a}{b}\].";
    let normalized = normalize(input, Options::ENABLE_MATH);

    assert_eq!(normalized, r"Inline ${x^2}$, then $$\frac{a}{b}$$.");
    assert_eq!(normalized.len(), input.len());
}

#[test]
fn leaves_unmatched_delimiters_and_code_literal() {
    let input = "Unmatched \\(x\n\n`\\(inline code\\)`\n\n```text\n\\[code block\\]\n```";

    assert_eq!(normalize(input, Options::ENABLE_MATH), input);
}

#[test]
fn normalizes_prose_around_code_without_touching_code() {
    let input = "`\\(code\\)` and \\(math\\)\n\n```text\n\\[code\\]\n```";

    assert_eq!(
        normalize(input, Options::ENABLE_MATH),
        "`\\(code\\)` and ${math}$\n\n```text\n\\[code\\]\n```"
    );
}

#[test]
fn flattens_multiline_display_math_without_changing_offsets() {
    let input = "before\n\n\\[\nL^2(P_X)\n=\n\\left\\{x\\right\\}\n\\]\n\nafter";
    let normalized = normalize(input, Options::ENABLE_MATH);

    assert_eq!(
        normalized,
        "before\n\n$$ L^2(P_X) = \\left\\{x\\right\\} $$\n\nafter"
    );
    assert_eq!(normalized.len(), input.len());
}

#[test]
fn flattens_native_display_math_but_not_code() {
    let input = "$$\nx\n=\ny\n$$\n\n`$$\ncode\n$$`\n\n```text\n$$\ncode\n$$\n```";

    assert_eq!(
        normalize(input, Options::ENABLE_MATH),
        "$$ x = y $$\n\n`$$\ncode\n$$`\n\n```text\n$$\ncode\n$$\n```"
    );
}
