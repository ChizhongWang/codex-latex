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
